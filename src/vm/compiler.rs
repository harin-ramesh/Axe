use crate::ast::{Expr, ExprKind, Literal, Operation, Program, Stmt, UnaryOp};
use crate::context::Context;

use super::bytecode::{Bytecode, BytecodeBuilder, Constant};
use super::instructions::Instruction;
use super::tables::{GlobalTable, LocalTable};

/// Compiles AST to bytecode
pub struct Compiler<'ctx> {
    builder: BytecodeBuilder,
    ctx: &'ctx Context,
    globals: GlobalTable,
    locals: LocalTable,
    scope_depth: usize,
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
            locals: LocalTable::new(),
            scope_depth: 0,
        }
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
                self.scope_depth += 1;
                for stmt in stmts {
                    self.compile_stmt(stmt);
                }
                let popped = self.locals.pop_scope(self.scope_depth);
                for _ in 0..popped {
                    self.builder.emit(Instruction::POP);
                }
                self.scope_depth -= 1;
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
                    if self.scope_depth == 0 {
                        let idx = self.globals.define(*symbol).expect("dup");
                        self.builder.emit(Instruction::DEFINE_GLOBAL);
                        self.builder.emit(idx);
                    } else {
                        self.locals.define(*symbol, self.scope_depth).expect("dup");
                        // no instruction — value already sits in this local's slot
                    }
                }
            }
            Stmt::Assign(symbol, expr) => {
                self.compile_expr(expr);
                if let Some(idx) = self.locals.resolve(*symbol, self.scope_depth) {
                    self.builder.emit(Instruction::SET_LOCAL);
                    self.builder.emit(idx);
                } else if let Some(idx) = self.globals.resolve(*symbol) {
                    self.builder.emit(Instruction::SET_GLOBAL);
                    self.builder.emit(idx);
                } else {
                    panic!("assignment to undefined variable");
                }
                self.builder.emit(Instruction::POP);
            }
            Stmt::Function(symbol, params, stmts) => {
                let jump_over_func = self.builder.emit_jump(Instruction::JUMP);
                let entry = self.builder.here();

                self.scope_depth += 1;
                for param in params {
                    self.locals
                        .define(*param, self.scope_depth)
                        .expect("dup param");
                }

                self.compile_stmt(stmts);
                self.builder.emit(Instruction::NULL);
                self.builder.emit(Instruction::RETURN);

                self.locals.pop_scope(self.scope_depth);
                self.scope_depth -= 1;

                self.builder.patch_jump(jump_over_func);

                self.builder.emit_constant(Constant::Fn {
                    entry,
                    arity: params.len() as u8,
                });

                if self.scope_depth == 0 {
                    let idx = self.globals.define(*symbol).expect("dup");
                    self.builder.emit(Instruction::DEFINE_GLOBAL);
                    self.builder.emit(idx);
                } else {
                    self.locals.define(*symbol, self.scope_depth).expect("dup");
                }
            }
            Stmt::Return(expr) => {
                self.compile_expr(expr);
                self.builder.emit(Instruction::RETURN);
            }
            // TODO: Implement other statements
            _ => todo!("Statement not yet implemented: {:?}", stmt),
        }
    }

    /// Compile a single expression and return the finished bytecode
    pub fn compile_expr_only(mut self, expr: &Expr) -> Bytecode {
        self.compile_expr(expr);
        self.builder.emit(Instruction::HALT);
        self.builder.build()
    }

    fn compile_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Literal(lit) => self.compile_literal(lit),
            ExprKind::Binary(op, lhs, rhs) => self.compile_binary(op, lhs, rhs),
            ExprKind::Unary(op, operand) => self.compile_unary(op, operand),
            ExprKind::Var(var) => {
                if let Some(idx) = self.locals.resolve(*var, self.scope_depth) {
                    self.builder.emit(Instruction::GET_LOCAL);
                    self.builder.emit(idx);
                } else if let Some(idx) = self.globals.resolve(*var) {
                    self.builder.emit(Instruction::GET_GLOBAL);
                    self.builder.emit(idx);
                } else {
                    panic!("undefined variable");
                }
            }
            ExprKind::Call(name, args) => {
                let idx = self.globals.resolve(*name).expect("undefined function");
                self.builder.emit(Instruction::GET_GLOBAL);
                self.builder.emit(idx);
                for arg in args {
                    self.compile_expr(arg);
                }
                self.builder.emit(Instruction::CALL);
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
        assert_eq!(bytecode.code, vec![Instruction::CONST, 0, Instruction::HALT]);

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
}
