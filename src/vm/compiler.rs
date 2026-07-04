use crate::Symbol;
use crate::ast::{Expr, ExprKind, Literal, Operation, ParamVec, Program, Stmt, UnaryOp};
use crate::context::Context;

use super::bytecode::{Bytecode, BytecodeBuilder, Constant};
use super::instructions::Instruction;
use super::tables::GlobalTable;

/// A local variable in a function scope. Its stack slot equals its index in the
/// scope's `locals` vector (locals are pushed/popped LIFO).
#[derive(Clone, Copy)]
struct Local {
    name: Symbol,
    depth: usize,
    /// Set once a nested function captures this local as an upvalue, so scope
    /// exit knows to emit `CLOSE_UPVALUE` instead of a plain `POP`.
    captured: bool,
}

/// Describes how a closure captures one upvalue: from the immediately enclosing
/// function's local (`is_local`), or by inheriting the enclosing closure's
/// upvalue.
#[derive(Clone, Copy)]
struct UpvalueDesc {
    index: u8,
    is_local: bool,
}

/// Compilation state for a single function (or the top-level script). Functions
/// nest, so the compiler holds a stack of these.
struct FnScope {
    locals: Vec<Local>,
    upvalues: Vec<UpvalueDesc>,
    scope_depth: usize,
}

impl FnScope {
    fn new() -> Self {
        Self {
            locals: Vec::new(),
            upvalues: Vec::new(),
            scope_depth: 0,
        }
    }
}

/// Where a name resolves to.
enum VarLoc {
    Local(u8),
    Upvalue(u8),
    Global(u8),
    Undefined,
}

/// Compiles AST to bytecode
pub struct Compiler<'ctx> {
    builder: BytecodeBuilder,
    ctx: &'ctx Context,
    globals: GlobalTable,
    /// Stack of function scopes; the last is the one being compiled. Index 0 is
    /// the top-level script.
    fn_scopes: Vec<FnScope>,
    /// Counter for generating unique names for compiler-synthesized locals
    /// (e.g. the hidden list/index locals of a `for` loop), so nested loops
    /// don't collide.
    synthetic_counter: usize,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(ctx: &'ctx Context) -> Self {
        let mut globals = GlobalTable::new();
        for (name, _) in super::builtins::builtins() {
            globals.define(ctx.intern(name)).expect("dup builtin");
        }

        Compiler {
            builder: BytecodeBuilder::new(),
            ctx,
            globals,
            fn_scopes: vec![FnScope::new()],
            synthetic_counter: 0,
        }
    }

    fn scope(&self) -> &FnScope {
        self.fn_scopes.last().unwrap()
    }

    fn scope_mut(&mut self) -> &mut FnScope {
        self.fn_scopes.last_mut().unwrap()
    }

    fn at_global(&self) -> bool {
        self.fn_scopes.len() == 1 && self.scope().scope_depth == 0
    }

    fn begin_scope(&mut self) {
        self.scope_mut().scope_depth += 1;
    }

    fn end_scope(&mut self) {
        let depth = self.scope().scope_depth;
        while let Some(&Local {
            depth: d, captured, ..
        }) = self.scope().locals.last()
        {
            if d < depth {
                break;
            }
            self.scope_mut().locals.pop();
            if captured {
                self.builder.emit(Instruction::CLOSE_UPVALUE);
            } else {
                self.builder.emit(Instruction::POP);
            }
        }
        self.scope_mut().scope_depth -= 1;
    }

    fn discard_scope_locals(&mut self) {
        let depth = self.scope().scope_depth;
        let s = self.scope_mut();
        while let Some(l) = s.locals.last() {
            if l.depth >= depth {
                s.locals.pop();
            } else {
                break;
            }
        }
        s.scope_depth -= 1;
    }

    fn add_local(&mut self, name: Symbol) -> u8 {
        let depth = self.scope().scope_depth;
        let s = self.scope_mut();
        let slot = s.locals.len() as u8;
        s.locals.push(Local {
            name,
            depth,
            captured: false,
        });
        slot
    }

    fn resolve_local_in(&self, scope_idx: usize, name: Symbol) -> Option<u8> {
        self.fn_scopes[scope_idx]
            .locals
            .iter()
            .rposition(|l| l.name == name)
            .map(|i| i as u8)
    }

    fn resolve_upvalue(&mut self, scope_idx: usize, name: Symbol) -> Option<u8> {
        if scope_idx == 0 {
            return None;
        }
        let enclosing = scope_idx - 1;
        if let Some(local) = self.resolve_local_in(enclosing, name) {
            self.fn_scopes[enclosing].locals[local as usize].captured = true;
            return Some(self.add_upvalue(scope_idx, local, true));
        }
        if let Some(uv) = self.resolve_upvalue(enclosing, name) {
            return Some(self.add_upvalue(scope_idx, uv, false));
        }
        None
    }

    fn add_upvalue(&mut self, scope_idx: usize, index: u8, is_local: bool) -> u8 {
        if let Some(i) = self.fn_scopes[scope_idx]
            .upvalues
            .iter()
            .position(|u| u.index == index && u.is_local == is_local)
        {
            return i as u8;
        }
        let ups = &mut self.fn_scopes[scope_idx].upvalues;
        ups.push(UpvalueDesc { index, is_local });
        (ups.len() - 1) as u8
    }

    fn resolve_variable(&mut self, name: Symbol) -> VarLoc {
        let top = self.fn_scopes.len() - 1;
        if let Some(slot) = self.resolve_local_in(top, name) {
            return VarLoc::Local(slot);
        }
        if let Some(uv) = self.resolve_upvalue(top, name) {
            return VarLoc::Upvalue(uv);
        }
        if let Some(idx) = self.globals.resolve(name) {
            return VarLoc::Global(idx);
        }
        VarLoc::Undefined
    }

    /// Compile a program and return the finished bytecode
    pub fn compile(mut self, program: &Program) -> Bytecode {
        for stmt in &program.stmts {
            self.compile_stmt(stmt);
        }
        self.builder.emit(Instruction::HALT);
        self.builder.build()
    }

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                self.compile_expr(expr);
                // Pop the result since it's an expression statement
                self.builder.emit(Instruction::POP);
            }
            Stmt::Block(stmts) => {
                self.begin_scope();
                for stmt in stmts {
                    self.compile_stmt(stmt);
                }
                self.end_scope();
            }
            Stmt::If(cond, then_stmt, else_stmt) => {
                self.compile_expr(cond);
                let jump_to_else = self.builder.emit_jump(Instruction::JUMP_IF_FALSE);
                self.compile_stmt(then_stmt);
                let jump_over_else = self.builder.emit_jump(Instruction::JUMP);
                self.builder.patch_jump(jump_to_else);
                self.compile_stmt(else_stmt);
                self.builder.patch_jump(jump_over_else);
            }
            Stmt::Let(bindings) => {
                for (symbol, init) in bindings {
                    match init {
                        Some(expr) => self.compile_expr(expr),
                        None => self.builder.emit(Instruction::NULL),
                    }
                    if self.at_global() {
                        let idx = self.globals.define(*symbol).expect("dup");
                        self.builder.emit(Instruction::DEFINE_GLOBAL);
                        self.builder.emit(idx);
                    } else {
                        self.add_local(*symbol);
                        // no instruction — value already sits in this local's slot
                    }
                }
            }
            Stmt::Assign(symbol, expr) => {
                self.compile_expr(expr);
                match self.resolve_variable(*symbol) {
                    VarLoc::Local(slot) => {
                        self.builder.emit(Instruction::SET_LOCAL);
                        self.builder.emit(slot);
                    }
                    VarLoc::Upvalue(slot) => {
                        self.builder.emit(Instruction::SET_UPVALUE);
                        self.builder.emit(slot);
                    }
                    VarLoc::Global(idx) => {
                        self.builder.emit(Instruction::SET_GLOBAL);
                        self.builder.emit(idx);
                    }
                    VarLoc::Undefined => panic!("assignment to undefined variable"),
                }
                self.builder.emit(Instruction::POP);
            }
            Stmt::Function(symbol, params, stmts) => {
                if self.at_global() {
                    let idx = self.globals.define(*symbol).expect("dup");
                    self.compile_function(params, stmts);
                    self.builder.emit(Instruction::DEFINE_GLOBAL);
                    self.builder.emit(idx);
                } else {
                    self.add_local(*symbol);
                    self.compile_function(params, stmts);
                }
            }
            Stmt::Class(name, parent, body) => self.compile_class(name, *parent, body),
            Stmt::PropertyAssign(obj_expr, prop, value_expr) => {
                self.compile_expr(obj_expr);
                self.compile_expr(value_expr);
                let name_const = self.builder.add_constant(Constant::Sym(*prop));
                self.builder.emit(Instruction::SET_PROPERTY);
                self.builder.emit(name_const);
                self.builder.emit(Instruction::POP);
            }
            Stmt::While(cond, body) => {
                // loop_start:
                //   <cond> ; JUMP_IF_FALSE exit ; <body> ; LOOP loop_start
                // exit:
                let loop_start = self.builder.here();
                self.compile_expr(cond);
                let exit_jump = self.builder.emit_jump(Instruction::JUMP_IF_FALSE);
                self.compile_stmt(body);
                self.builder.emit_loop(loop_start);
                self.builder.patch_jump(exit_jump);
            }
            Stmt::For(var, iterable, body) => self.compile_for(var, iterable, body),
            Stmt::Return(expr) => {
                self.compile_expr(expr);
                self.builder.emit(Instruction::RETURN);
            }
            _ => todo!("Statement not yet implemented: {:?}", stmt),
        }
    }

    /// Compile `for var in iterable { body }` by desugaring to an index loop
    /// over the (list) iterable, using three hidden locals: the list, the
    /// index, and the loop variable. Wrapped in its own scope so the loop
    /// variables are locals even at top level.
    fn compile_for(&mut self, var: &Symbol, iterable: &Expr, body: &Stmt) {
        self.begin_scope();

        // Unique names so nested `for` loops don't collide in the flat table.
        let uid = self.synthetic_counter;
        self.synthetic_counter += 1;
        let list_name = self.ctx.intern(&format!("$for_list{}", uid));
        let idx_name = self.ctx.intern(&format!("$for_idx{}", uid));

        // hidden: __list = iterable  (value stays in this local's slot)
        self.compile_expr(iterable);
        let list_slot = self.add_local(list_name);

        // hidden: __idx = 0
        self.builder.emit_constant(Constant::Int(0));
        let idx_slot = self.add_local(idx_name);

        // loop variable, seeded with a placeholder so it owns a stack slot
        self.builder.emit(Instruction::NULL);
        let var_slot = self.add_local(*var);

        // loop_start:  if !(idx < len(list)) goto exit
        let loop_start = self.builder.here();
        self.builder.emit(Instruction::GET_LOCAL);
        self.builder.emit(idx_slot);
        self.builder.emit(Instruction::GET_LOCAL);
        self.builder.emit(list_slot);
        self.builder.emit(Instruction::LEN);
        self.builder.emit(Instruction::LT);
        let exit_jump = self.builder.emit_jump(Instruction::JUMP_IF_FALSE);

        // var = list[idx]
        self.builder.emit(Instruction::GET_LOCAL);
        self.builder.emit(list_slot);
        self.builder.emit(Instruction::GET_LOCAL);
        self.builder.emit(idx_slot);
        self.builder.emit(Instruction::GET_INDEX);
        self.builder.emit(Instruction::SET_LOCAL);
        self.builder.emit(var_slot);
        self.builder.emit(Instruction::POP);

        self.compile_stmt(body);

        // idx = idx + 1
        self.builder.emit(Instruction::GET_LOCAL);
        self.builder.emit(idx_slot);
        self.builder.emit_constant(Constant::Int(1));
        self.builder.emit(Instruction::ADD);
        self.builder.emit(Instruction::SET_LOCAL);
        self.builder.emit(idx_slot);
        self.builder.emit(Instruction::POP);

        self.builder.emit_loop(loop_start);
        self.builder.patch_jump(exit_jump);

        // Discard the three hidden locals (var, idx, list).
        self.end_scope();
    }

    fn compile_function(&mut self, params: &ParamVec, body: &Stmt) {
        let jump_over = self.builder.emit_jump(Instruction::JUMP);
        let entry = self.builder.here();

        self.fn_scopes.push(FnScope::new());
        for param in params {
            self.add_local(*param);
        }

        self.compile_function_body(body);

        let scope = self.fn_scopes.pop().unwrap();

        self.builder.patch_jump(jump_over);

        let arity = params.len() as u8;
        if scope.upvalues.is_empty() {
            // Non-capturing: a flat function value, no heap allocation.
            self.builder.emit_constant(Constant::Fn { entry, arity });
        } else {
            // Capturing: emit CLOSURE with the capture descriptors.
            let fn_const = self.builder.add_constant(Constant::Fn { entry, arity });
            self.builder.emit(Instruction::CLOSURE);
            self.builder.emit(fn_const);
            self.builder.emit(scope.upvalues.len() as u8);
            for uv in &scope.upvalues {
                self.builder.emit(uv.is_local as u8);
                self.builder.emit(uv.index);
            }
        }
    }

    fn compile_function_body(&mut self, body: &Stmt) {
        if let Stmt::Block(stmts) = body {
            self.scope_mut().scope_depth += 1;
            for stmt in stmts {
                self.compile_stmt(stmt);
            }
            self.discard_scope_locals();
        } else {
            self.compile_stmt(body);
        }

        // Fall-through with no `return`: yield null.
        self.builder.emit(Instruction::NULL);
        self.builder.emit(Instruction::RETURN);
    }

    fn compile_class(&mut self, name: &Symbol, parent: Option<Symbol>, body: &[Stmt]) {
        if !self.at_global() {
            todo!("local class declarations are not yet supported");
        }

        let class_idx = self.globals.define(*name).expect("dup class");

        let name_const = self.builder.add_constant(Constant::Sym(*name));
        self.builder.emit(Instruction::CLASS);
        self.builder.emit(name_const);

        if let Some(parent) = parent {
            let idx = self
                .globals
                .resolve(parent)
                .expect("undefined parent class");
            self.builder.emit(Instruction::GET_GLOBAL);
            self.builder.emit(idx);
            self.builder.emit(Instruction::INHERIT);
        }

        for member in body {
            match member {
                Stmt::Let(bindings) => {
                    for (sym, init) in bindings {
                        match init {
                            Some(expr) => self.compile_expr(expr),
                            None => self.builder.emit(Instruction::NULL),
                        }
                        let c = self.builder.add_constant(Constant::Sym(*sym));
                        self.builder.emit(Instruction::STATIC_FIELD);
                        self.builder.emit(c);
                    }
                }
                Stmt::Function(fn_name, params, fn_body) => {
                    self.compile_function(params, fn_body);
                    let c = self.builder.add_constant(Constant::Sym(*fn_name));
                    self.builder.emit(Instruction::METHOD);
                    self.builder.emit(c);
                }
                _ => {}
            }
        }

        self.builder.emit(Instruction::DEFINE_GLOBAL);
        self.builder.emit(class_idx);
    }

    pub fn compile_expr_only(mut self, expr: &Expr) -> Bytecode {
        self.compile_expr(expr);
        self.builder.emit(Instruction::HALT);
        self.builder.build()
    }

    fn compile_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Literal(lit) => self.compile_literal(lit),
            ExprKind::List(elements) => {
                for element in elements {
                    self.compile_expr(element);
                }
                self.builder.emit(Instruction::BUILD_LIST);
                self.builder.emit(elements.len() as u8);
            }
            ExprKind::Binary(op, lhs, rhs) => self.compile_binary(op, lhs, rhs),
            ExprKind::Unary(op, operand) => self.compile_unary(op, operand),
            ExprKind::Var(var) => match self.resolve_variable(*var) {
                VarLoc::Local(slot) => {
                    self.builder.emit(Instruction::GET_LOCAL);
                    self.builder.emit(slot);
                }
                VarLoc::Upvalue(slot) => {
                    self.builder.emit(Instruction::GET_UPVALUE);
                    self.builder.emit(slot);
                }
                VarLoc::Global(idx) => {
                    self.builder.emit(Instruction::GET_GLOBAL);
                    self.builder.emit(idx);
                }
                VarLoc::Undefined => panic!("undefined variable"),
            },
            ExprKind::Call(name, args) => {
                // The callee may be a global, a local, or a captured upvalue.
                match self.resolve_variable(*name) {
                    VarLoc::Local(slot) => {
                        self.builder.emit(Instruction::GET_LOCAL);
                        self.builder.emit(slot);
                    }
                    VarLoc::Upvalue(slot) => {
                        self.builder.emit(Instruction::GET_UPVALUE);
                        self.builder.emit(slot);
                    }
                    VarLoc::Global(idx) => {
                        self.builder.emit(Instruction::GET_GLOBAL);
                        self.builder.emit(idx);
                    }
                    VarLoc::Undefined => panic!("undefined function"),
                }
                for arg in args {
                    self.compile_expr(arg);
                }
                self.builder.emit(Instruction::CALL);
                self.builder.emit(args.len() as u8);
            }
            // new ClassName(args...)
            ExprKind::New(class, args) => {
                let idx = self.globals.resolve(*class).expect("undefined class");
                self.builder.emit(Instruction::GET_GLOBAL);
                self.builder.emit(idx);
                for arg in args {
                    self.compile_expr(arg);
                }
                let init_const = self
                    .builder
                    .add_constant(Constant::Sym(self.ctx.intern("init")));
                self.builder.emit(Instruction::NEW);
                self.builder.emit(init_const);
                self.builder.emit(args.len() as u8);
            }
            // obj.property
            ExprKind::Property(obj, name) => {
                self.compile_expr(obj);
                let c = self.builder.add_constant(Constant::Sym(*name));
                self.builder.emit(Instruction::GET_PROPERTY);
                self.builder.emit(c);
            }
            // obj.method(args...)
            ExprKind::MethodCall(obj, method, args) => {
                self.compile_expr(obj);
                for arg in args {
                    self.compile_expr(arg);
                }
                let c = self.builder.add_constant(Constant::Sym(*method));
                self.builder.emit(Instruction::INVOKE);
                self.builder.emit(c);
                self.builder.emit(args.len() as u8);
            }
            // Class::property
            ExprKind::StaticProperty(obj, name) => {
                self.compile_expr(obj);
                let c = self.builder.add_constant(Constant::Sym(*name));
                self.builder.emit(Instruction::GET_STATIC);
                self.builder.emit(c);
            }
            // Class::method(args...)
            ExprKind::StaticMethodCall(obj, method, args) => {
                self.compile_expr(obj);
                for arg in args {
                    self.compile_expr(arg);
                }
                let c = self.builder.add_constant(Constant::Sym(*method));
                self.builder.emit(Instruction::STATIC_INVOKE);
                self.builder.emit(c);
                self.builder.emit(args.len() as u8);
            }
            // TODO: Implement other expressions
            _ => todo!("Expression not yet implemented: {:?}", expr),
        }
    }

    fn compile_literal(&mut self, lit: &Literal) {
        match lit {
            Literal::Null => self.builder.emit(Instruction::NULL),
            Literal::Bool(true) => self.builder.emit(Instruction::TRUE),
            Literal::Bool(false) => self.builder.emit(Instruction::FALSE),
            Literal::Int(n) => self.builder.emit_constant(Constant::Int(*n)),
            Literal::Float(n) => self.builder.emit_constant(Constant::Float(*n)),
            Literal::Str(s) => {
                let string = self.ctx.resolve(*s);
                self.builder.emit_constant(Constant::Str(string.into()))
            }
        }
    }

    fn compile_binary(&mut self, op: &Operation, lhs: &Expr, rhs: &Expr) {
        // Constant folding: if both operands reduce to compile-time constants,
        // evaluate the operation now and emit a single constant. Because
        // `fold_const` recurses, a fully-constant tree collapses at the outermost
        // operation (e.g. `(10 + 20) * 2 - 5` becomes a single `55`).
        if let (Some(a), Some(b)) = (fold_const(lhs), fold_const(rhs)) {
            if let Some(folded) = fold_binary(op, a, b) {
                self.compile_literal(&folded);
                return;
            }
        }

        // Compile left operand first, then right
        self.compile_expr(lhs);
        self.compile_expr(rhs);

        // Emit the appropriate instruction
        let instruction = match op {
            Operation::Add => Instruction::ADD,
            Operation::Sub => Instruction::SUB,
            Operation::Mul => Instruction::MUL,
            Operation::Div => Instruction::DIV,
            Operation::Mod => Instruction::MOD,
            Operation::Gt => Instruction::GT,
            Operation::Lt => Instruction::LT,
            Operation::Gte => Instruction::GTE,
            Operation::Lte => Instruction::LTE,
            Operation::Eq => Instruction::EQ,
            Operation::Neq => Instruction::NEQ,
            Operation::And => Instruction::AND,
            Operation::Or => Instruction::OR,
            Operation::BitwiseAnd => Instruction::BITAND,
            Operation::BitwiseOr => Instruction::BITOR,
        };
        self.builder.emit(instruction);
    }

    fn compile_unary(&mut self, op: &UnaryOp, operand: &Expr) {
        // Constant folding for unary operations (e.g. `-42`, `~0`, `!true`).
        if let Some(v) = fold_const(operand) {
            if let Some(folded) = fold_unary(op, v) {
                self.compile_literal(&folded);
                return;
            }
        }

        self.compile_expr(operand);

        let instruction = match op {
            UnaryOp::Neg => Instruction::NEG,
            UnaryOp::Not => Instruction::NOT,
            UnaryOp::Inv => Instruction::BITINV,
        };
        self.builder.emit(instruction);
    }
}

/// Recursively evaluate an expression built entirely from literal operands.
///
/// Returns `None` for anything that isn't a compile-time constant — variables,
/// calls, string operands, or operations whose result can't be folded *safely*
/// (integer overflow, division by zero). Leaving those unfolded means the VM
/// reproduces the exact same runtime behavior it had before folding.
fn fold_const(expr: &Expr) -> Option<Literal> {
    match &expr.kind {
        ExprKind::Literal(lit) => Some(*lit),
        ExprKind::Unary(op, operand) => fold_unary(op, fold_const(operand)?),
        ExprKind::Binary(op, lhs, rhs) => fold_binary(op, fold_const(lhs)?, fold_const(rhs)?),
        _ => None,
    }
}

fn fold_unary(op: &UnaryOp, v: Literal) -> Option<Literal> {
    match (op, v) {
        // `checked_neg` guards i64::MIN, which would overflow.
        (UnaryOp::Neg, Literal::Int(n)) => n.checked_neg().map(Literal::Int),
        (UnaryOp::Neg, Literal::Float(f)) => Some(Literal::Float(-f)),
        (UnaryOp::Not, Literal::Bool(b)) => Some(Literal::Bool(!b)),
        (UnaryOp::Inv, Literal::Int(n)) => Some(Literal::Int(!n)),
        _ => None,
    }
}

fn fold_binary(op: &Operation, a: Literal, b: Literal) -> Option<Literal> {
    use Literal::{Bool, Float, Int};
    use Operation::*;
    match (op, a, b) {
        // Integer arithmetic. `checked_*` returns `None` on overflow and on
        // div/rem by zero (and i64::MIN / -1), so those stay unfolded and the
        // VM panics at runtime exactly as it would have.
        (Add, Int(x), Int(y)) => x.checked_add(y).map(Int),
        (Sub, Int(x), Int(y)) => x.checked_sub(y).map(Int),
        (Mul, Int(x), Int(y)) => x.checked_mul(y).map(Int),
        (Div, Int(x), Int(y)) => x.checked_div(y).map(Int),
        (Mod, Int(x), Int(y)) => x.checked_rem(y).map(Int),

        // Float arithmetic. Division by zero yields inf/NaN — the same result
        // the VM produces — so it's safe to fold.
        (Add, Float(x), Float(y)) => Some(Float(x + y)),
        (Sub, Float(x), Float(y)) => Some(Float(x - y)),
        (Mul, Float(x), Float(y)) => Some(Float(x * y)),
        (Div, Float(x), Float(y)) => Some(Float(x / y)),
        (Mod, Float(x), Float(y)) => Some(Float(x % y)),

        // Comparisons (same-type operands only, matching VM semantics).
        (Gt, Int(x), Int(y)) => Some(Bool(x > y)),
        (Lt, Int(x), Int(y)) => Some(Bool(x < y)),
        (Gte, Int(x), Int(y)) => Some(Bool(x >= y)),
        (Lte, Int(x), Int(y)) => Some(Bool(x <= y)),
        (Gt, Float(x), Float(y)) => Some(Bool(x > y)),
        (Lt, Float(x), Float(y)) => Some(Bool(x < y)),
        (Gte, Float(x), Float(y)) => Some(Bool(x >= y)),
        (Lte, Float(x), Float(y)) => Some(Bool(x <= y)),

        // Equality (same-type only; mixed types fall through to the runtime).
        (Eq, Int(x), Int(y)) => Some(Bool(x == y)),
        (Neq, Int(x), Int(y)) => Some(Bool(x != y)),
        (Eq, Float(x), Float(y)) => Some(Bool(x == y)),
        (Neq, Float(x), Float(y)) => Some(Bool(x != y)),
        (Eq, Bool(x), Bool(y)) => Some(Bool(x == y)),
        (Neq, Bool(x), Bool(y)) => Some(Bool(x != y)),

        // Logical (both operands boolean).
        (And, Bool(x), Bool(y)) => Some(Bool(x && y)),
        (Or, Bool(x), Bool(y)) => Some(Bool(x || y)),

        // Bitwise.
        (BitwiseAnd, Int(x), Int(y)) => Some(Int(x & y)),
        (BitwiseOr, Int(x), Int(y)) => Some(Int(x | y)),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use crate::vm::{AxeVM, Value};

    fn compile_and_run(ctx: &Context, expr: Expr) -> Option<Value> {
        let compiler = Compiler::new(ctx);
        let bytecode = compiler.compile_expr_only(&expr);
        let mut vm = AxeVM::new(&bytecode);
        vm.exec()
    }

    /// Run an expression and render its result via the VM's heap. Needed for
    /// string results, whose `Value`s are opaque `ObjRef` handles.
    fn compile_and_display(ctx: &Context, expr: Expr) -> Option<String> {
        let compiler = Compiler::new(ctx);
        let bytecode = compiler.compile_expr_only(&expr);
        let mut vm = AxeVM::new(&bytecode);
        let result = vm.exec();
        result.map(|v| vm.display_value(&v))
    }

    #[test]
    fn test_compile_literals() {
        let ctx = Context::new();

        // Null
        assert_eq!(
            compile_and_run(&ctx, Expr::Literal(Literal::Null)),
            Some(Value::Null)
        );

        // Bool
        assert_eq!(
            compile_and_run(&ctx, Expr::Literal(Literal::Bool(true))),
            Some(Value::Bool(true))
        );

        // Int
        assert_eq!(
            compile_and_run(&ctx, Expr::Literal(Literal::Int(42))),
            Some(Value::Int(42))
        );

        // Float
        assert_eq!(
            compile_and_run(&ctx, Expr::Literal(Literal::Float(3.14))),
            Some(Value::Float(3.14))
        );

        // String
        let hello_sym = ctx.intern("hello");
        assert_eq!(
            compile_and_display(&ctx, Expr::Literal(Literal::Str(hello_sym))),
            Some("hello".to_string())
        );
    }

    #[test]
    fn test_compile_binary_arithmetic() {
        let ctx = Context::new();

        // 10 + 20
        let expr = Expr::Binary(
            Operation::Add,
            Box::new(Expr::Literal(Literal::Int(10))),
            Box::new(Expr::Literal(Literal::Int(20))),
        );
        assert_eq!(compile_and_run(&ctx, expr), Some(Value::Int(30)));

        // 50 - 20
        let expr = Expr::Binary(
            Operation::Sub,
            Box::new(Expr::Literal(Literal::Int(50))),
            Box::new(Expr::Literal(Literal::Int(20))),
        );
        assert_eq!(compile_and_run(&ctx, expr), Some(Value::Int(30)));

        // 6 * 7
        let expr = Expr::Binary(
            Operation::Mul,
            Box::new(Expr::Literal(Literal::Int(6))),
            Box::new(Expr::Literal(Literal::Int(7))),
        );
        assert_eq!(compile_and_run(&ctx, expr), Some(Value::Int(42)));

        // 100 / 4
        let expr = Expr::Binary(
            Operation::Div,
            Box::new(Expr::Literal(Literal::Int(100))),
            Box::new(Expr::Literal(Literal::Int(4))),
        );
        assert_eq!(compile_and_run(&ctx, expr), Some(Value::Int(25)));

        // 17 % 5
        let expr = Expr::Binary(
            Operation::Mod,
            Box::new(Expr::Literal(Literal::Int(17))),
            Box::new(Expr::Literal(Literal::Int(5))),
        );
        assert_eq!(compile_and_run(&ctx, expr), Some(Value::Int(2)));
    }

    #[test]
    fn test_compile_binary_comparison() {
        let ctx = Context::new();

        // 5 > 3
        let expr = Expr::Binary(
            Operation::Gt,
            Box::new(Expr::Literal(Literal::Int(5))),
            Box::new(Expr::Literal(Literal::Int(3))),
        );
        assert_eq!(compile_and_run(&ctx, expr), Some(Value::Bool(true)));

        // 5 < 3
        let expr = Expr::Binary(
            Operation::Lt,
            Box::new(Expr::Literal(Literal::Int(5))),
            Box::new(Expr::Literal(Literal::Int(3))),
        );
        assert_eq!(compile_and_run(&ctx, expr), Some(Value::Bool(false)));

        // 5 == 5
        let expr = Expr::Binary(
            Operation::Eq,
            Box::new(Expr::Literal(Literal::Int(5))),
            Box::new(Expr::Literal(Literal::Int(5))),
        );
        assert_eq!(compile_and_run(&ctx, expr), Some(Value::Bool(true)));

        // 5 != 3
        let expr = Expr::Binary(
            Operation::Neq,
            Box::new(Expr::Literal(Literal::Int(5))),
            Box::new(Expr::Literal(Literal::Int(3))),
        );
        assert_eq!(compile_and_run(&ctx, expr), Some(Value::Bool(true)));
    }

    #[test]
    fn test_compile_unary() {
        let ctx = Context::new();

        // -42
        let expr = Expr::Unary(UnaryOp::Neg, Box::new(Expr::Literal(Literal::Int(42))));
        assert_eq!(compile_and_run(&ctx, expr), Some(Value::Int(-42)));

        // !true
        let expr = Expr::Unary(UnaryOp::Not, Box::new(Expr::Literal(Literal::Bool(true))));
        assert_eq!(compile_and_run(&ctx, expr), Some(Value::Bool(false)));
    }

    #[test]
    fn test_compile_complex_expression() {
        let ctx = Context::new();

        // (10 + 20) * 2 - 5 = 55
        let expr = Expr::Binary(
            Operation::Sub,
            Box::new(Expr::Binary(
                Operation::Mul,
                Box::new(Expr::Binary(
                    Operation::Add,
                    Box::new(Expr::Literal(Literal::Int(10))),
                    Box::new(Expr::Literal(Literal::Int(20))),
                )),
                Box::new(Expr::Literal(Literal::Int(2))),
            )),
            Box::new(Expr::Literal(Literal::Int(5))),
        );
        assert_eq!(compile_and_run(&ctx, expr), Some(Value::Int(55)));
    }

    #[test]
    fn test_constant_folding_collapses_tree() {
        let ctx = Context::new();

        // (10 + 20) * 2 - 5 is fully constant and folds to a single 55.
        let expr = Expr::Binary(
            Operation::Sub,
            Box::new(Expr::Binary(
                Operation::Mul,
                Box::new(Expr::Binary(
                    Operation::Add,
                    Box::new(Expr::Literal(Literal::Int(10))),
                    Box::new(Expr::Literal(Literal::Int(20))),
                )),
                Box::new(Expr::Literal(Literal::Int(2))),
            )),
            Box::new(Expr::Literal(Literal::Int(5))),
        );

        let bytecode = Compiler::new(&ctx).compile_expr_only(&expr);

        // Whole tree collapses: one constant, and code is just CONST 0; HALT.
        assert_eq!(bytecode.constants, vec![Constant::Int(55)]);
        assert_eq!(
            bytecode.code,
            vec![Instruction::CONST, 0, Instruction::HALT]
        );

        // ...and it still evaluates correctly.
        let mut vm = AxeVM::new(&bytecode);
        assert_eq!(vm.exec(), Some(Value::Int(55)));
    }

    #[test]
    fn test_constant_folding_skips_div_by_zero() {
        let ctx = Context::new();

        // 1 / 0 must NOT be folded — the DIV opcode stays so the VM panics
        // at runtime exactly as it would without folding.
        let expr = Expr::Binary(
            Operation::Div,
            Box::new(Expr::Literal(Literal::Int(1))),
            Box::new(Expr::Literal(Literal::Int(0))),
        );

        let bytecode = Compiler::new(&ctx).compile_expr_only(&expr);
        assert!(bytecode.code.contains(&Instruction::DIV));
    }

    #[test]
    fn test_compile_string_concat() {
        let ctx = Context::new();

        // "Hello, " + "World!"
        let hello_sym = ctx.intern("Hello, ");
        let world_sym = ctx.intern("World!");
        let expr = Expr::Binary(
            Operation::Add,
            Box::new(Expr::Literal(Literal::Str(hello_sym))),
            Box::new(Expr::Literal(Literal::Str(world_sym))),
        );
        assert_eq!(
            compile_and_display(&ctx, expr),
            Some("Hello, World!".to_string())
        );
    }

    /// Compile and run a whole program from source, returning the value of its
    /// final (expression) statement rendered to a display string. Unlike the
    /// public `compile()`, the trailing expression's value is left on the stack
    /// instead of being popped, so tests can observe it.
    fn run_source(src: &str) -> Option<String> {
        let ctx = Context::new();
        let program = crate::parser::Parser::new(src, &ctx)
            .parse()
            .expect("parse failed");

        let mut compiler = Compiler::new(&ctx);
        let (last, rest) = program.stmts.split_last().expect("empty program");
        for stmt in rest {
            compiler.compile_stmt(stmt);
        }
        match last {
            Stmt::Expr(expr) => compiler.compile_expr(expr),
            other => compiler.compile_stmt(other),
        }
        compiler.builder.emit(Instruction::HALT);
        let bytecode = compiler.builder.build();

        let mut vm = AxeVM::new(&bytecode);
        vm.exec().map(|v| vm.display_value(&v))
    }

    #[test]
    fn test_class_fields_and_methods() {
        // Instantiation, `init`, property get/set, and implicit last-expr return.
        let out = run_source(
            "class Counter {
                fn init(self, start) { self.count = start; }
                fn increment(self) { self.count = self.count + 1; return self.count; }
                fn get(self) { return self.count; }
            }
            let c = new Counter(10);
            c.increment();
            c.increment();
            c.get();",
        );
        assert_eq!(out, Some("12".to_string()));
    }

    #[test]
    fn test_class_static_property_and_method() {
        assert_eq!(
            run_source(
                "class MathUtils {
                    let PI = 3;
                    fn add(a, b) { return a + b; }
                }
                MathUtils::add(MathUtils::PI, 39);"
            ),
            Some("42".to_string())
        );
    }

    #[test]
    fn test_class_inheritance() {
        // Child inherits parent's `init` and `speak`; its own method calls the
        // inherited one through `self`.
        let out = run_source(
            "class Animal {
                fn init(self, name) { self.name = name; }
                fn speak(self) { return self.name; }
            }
            class Dog : Animal {
                fn bark(self) { return self.speak(); }
            }
            let d = new Dog(\"Rex\");
            d.bark();",
        );
        assert_eq!(out, Some("Rex".to_string()));
    }

    #[test]
    fn test_closure_captures_param() {
        // adder captures make_adder's parameter x.
        let out = run_source(
            "fn make_adder(x) {
                 fn adder(y) { return x + y; }
                 return adder;
             }
             let add5 = make_adder(5);
             add5(10);",
        );
        assert_eq!(out, Some("15".to_string()));
    }

    #[test]
    fn test_closure_shared_mutable_capture() {
        // Repeated calls share and mutate the captured `c` (closed upvalue).
        let out = run_source(
            "fn counter() {
                 let c = 0;
                 fn inc() { c = c + 1; return c; }
                 return inc;
             }
             let f = counter();
             f(); f();
             f();",
        );
        assert_eq!(out, Some("3".to_string()));
    }

    #[test]
    fn test_closure_transitive_capture() {
        // inner captures `a` transitively through middle's upvalue (is_local=false).
        let out = run_source(
            "fn outer(a) {
                 fn middle(b) {
                     fn inner(c) { return a + b + c; }
                     return inner;
                 }
                 return middle;
             }
             let m = outer(100);
             let i = m(20);
             i(3);",
        );
        assert_eq!(out, Some("123".to_string()));
    }

    #[test]
    fn test_closures_are_independent() {
        // Two counters must not share state.
        let out = run_source(
            "fn counter() {
                 let c = 0;
                 fn inc() { c = c + 1; return c; }
                 return inc;
             }
             let a = counter();
             let b = counter();
             a(); a(); a();
             b();",
        );
        assert_eq!(out, Some("1".to_string())); // b is independent of a
    }

    #[test]
    fn test_while_loop() {
        // Sum 1..=100 with a while loop.
        let out = run_source(
            "let i = 1; let sum = 0;
             while (i <= 100) { sum = sum + i; i = i + 1; }
             sum;",
        );
        assert_eq!(out, Some("5050".to_string()));
    }

    #[test]
    fn test_for_over_list_literal() {
        let out = run_source(
            "let total = 0;
             for x in [10, 20, 30, 40] { total = total + x; }
             total;",
        );
        assert_eq!(out, Some("100".to_string()));
    }

    #[test]
    fn test_for_over_range() {
        let out = run_source(
            "let count = 0;
             for n in range(0, 1000) { count = count + n; }
             count;",
        );
        assert_eq!(out, Some("499500".to_string()));
    }

    #[test]
    fn test_nested_for_loops() {
        let out = run_source(
            "let grid = 0;
             for a in range(0, 3) { for b in range(0, 4) { grid = grid + 1; } }
             grid;",
        );
        assert_eq!(out, Some("12".to_string()));
    }

    #[test]
    fn test_list_literal_and_len_and_index() {
        assert_eq!(run_source("len([1, 2, 3, 4]);"), Some("4".to_string()));
        assert_eq!(
            run_source("[10, 20, 30];"),
            Some("[10, 20, 30]".to_string())
        );
        // range with explicit bounds
        assert_eq!(run_source("range(2, 5);"), Some("[2, 3, 4]".to_string()));
    }

    #[test]
    fn test_while_with_function_call_body() {
        // Loop body that calls a function, exercising loop + call interaction.
        let out = run_source(
            "fn sq(n) { return n * n; }
             let i = 0; let acc = 0;
             while (i < 5) { acc = acc + sq(i); i = i + 1; }
             acc;",
        );
        assert_eq!(out, Some("30".to_string())); // 0+1+4+9+16
    }

    #[test]
    fn test_class_without_init() {
        // A class with no constructor still instantiates; fields set later.
        let out = run_source(
            "class Box {
                fn put(self, v) { self.v = v; }
                fn get(self) { return self.v; }
            }
            let b = new Box();
            b.put(7);
            b.get();",
        );
        assert_eq!(out, Some("7".to_string()));
    }
}
