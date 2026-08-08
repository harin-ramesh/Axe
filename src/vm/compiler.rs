use crate::Symbol;
use crate::ast::{Expr, ExprKind, Literal, Operation, ParamVec, Program, Stmt, UnaryOp};
use crate::context::Context;
use crate::parser::Parser;

use fxhash::FxHashSet;

use super::bytecode::{Bytecode, BytecodeBuilder, Constant};
use super::instructions::Instruction;
use super::tables::GlobalTable;

use std::path::PathBuf;

pub trait ModuleLoader {
    fn load(&self, name: &str) -> Result<String, String>;
}

pub struct FileLoader {
    pub root: PathBuf,
}

impl ModuleLoader for FileLoader {
    fn load(&self, name: &str) -> Result<String, String> {
        let path = self.root.join(name).with_extension("ax");
        std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read '{}': {}", path.display(), e))
    }
}

#[derive(Debug, Clone)]
pub struct CompileError {
    pub message: String,
    pub line: u32,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line != 0 {
            write!(f, "[line {}] {}", self.line, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for CompileError {}

#[derive(Clone, Copy)]
struct Local {
    name: Symbol,
    depth: usize,
    captured: bool,
}

#[derive(Clone, Copy)]
struct UpvalueDesc {
    index: u8,
    is_local: bool,
}

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

enum VarLoc {
    Local(u8),
    Upvalue(u8),
    Global(u8),
    Undefined,
}

pub struct LoopScope {
    break_jumps: Vec<usize>,
    continue_jumps: Vec<usize>,
    break_depth: usize,
    continue_depth: usize,
}

pub struct Compiler<'ctx> {
    builder: BytecodeBuilder,
    ctx: &'ctx Context,
    globals: GlobalTable,
    fn_scopes: Vec<FnScope>,
    loop_scopes: Vec<LoopScope>,
    synthetic_counter: usize,
    line: u32,

    loader: Box<dyn ModuleLoader>,
    module_prefix: Option<String>,
    loaded: FxHashSet<Symbol>,
    loading: Vec<Symbol>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(ctx: &'ctx Context) -> Self {
        Self::with_root(ctx, ".")
    }

    /// `root` is the directory module names resolve against — normally the
    /// directory of the file being compiled.
    pub fn with_root(ctx: &'ctx Context, root: impl Into<PathBuf>) -> Self {
        Self::with_loader(ctx, Box::new(FileLoader { root: root.into() }))
    }

    pub fn with_loader(ctx: &'ctx Context, loader: Box<dyn ModuleLoader>) -> Self {
        let mut globals = GlobalTable::new();
        for (name, _) in super::builtins::builtins() {
            globals.define(ctx.intern(name)).expect("dup builtin");
        }

        Compiler {
            builder: BytecodeBuilder::new(),
            ctx,
            globals,
            fn_scopes: vec![FnScope::new()],
            loop_scopes: vec![],
            synthetic_counter: 0,
            line: 0,
            loader,
            module_prefix: None,
            loaded: FxHashSet::default(),
            loading: Vec::new(),
        }
    }

    fn err(&self, message: impl Into<String>) -> CompileError {
        CompileError {
            message: message.into(),
            line: self.line,
        }
    }

    fn mark_line(&mut self, line: u32) {
        if line != 0 {
            self.line = line;
            self.builder.set_line(line);
        }
    }

    fn name_of(&self, sym: Symbol) -> String {
        self.ctx.resolve(sym)
    }

    /// `("math", add)` -> the symbol `math$add`. `$` is not a legal identifier
    /// character, so a qualified name can never collide with one a user typed.
    fn qualified(&self, module: &str, name: Symbol) -> Symbol {
        self.ctx
            .intern(&format!("{}${}", module, self.name_of(name)))
    }

    /// Apply the module prefix currently in effect, if any.
    fn qualify(&self, name: Symbol) -> Symbol {
        match &self.module_prefix {
            Some(prefix) => self.qualified(prefix, name),
            None => name,
        }
    }

    /// Define a global in the namespace currently being compiled.
    fn define_global(&mut self, name: Symbol) -> Result<u8, CompileError> {
        let name = self.qualify(name);
        self.globals.define_or_get(name).map_err(|e| self.err(e))
    }

    /// Look a global up in the current namespace, falling back to the
    /// unprefixed table — which is how a module still sees `print` and `len`.
    fn resolve_global(&self, name: Symbol) -> Option<u8> {
        self.globals
            .resolve(self.qualify(name))
            .or_else(|| self.globals.resolve(name))
    }

    fn sym_const(&mut self, sym: Symbol) -> Result<u8, CompileError> {
        let c = self
            .builder
            .try_add_constant(Constant::Sym(sym))
            .map_err(|e| self.err(e))?;
        self.builder.name_sym(sym, self.name_of(sym));
        Ok(c)
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

    fn emit_pops_to_depth(&mut self, target: usize) {
        for i in (0..self.scope().locals.len()).rev() {
            let local = self.scope().locals[i];
            if local.depth <= target {
                break;
            }
            if local.captured {
                self.builder.emit(Instruction::CLOSE_UPVALUE);
            } else {
                self.builder.emit(Instruction::POP);
            }
        }
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
        if let Some(idx) = self.resolve_global(name) {
            return VarLoc::Global(idx);
        }
        VarLoc::Undefined
    }

    /// Compile a program and return the finished bytecode
    pub fn compile(mut self, program: &Program) -> Result<Bytecode, CompileError> {
        for stmt in &program.stmts {
            self.compile_stmt(stmt)?;
        }
        self.builder.emit(Instruction::HALT);
        Ok(self.builder.build())
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), CompileError> {
        match stmt {
            Stmt::Expr(expr) => {
                self.compile_expr(expr)?;
                // Pop the result since it's an expression statement
                self.builder.emit(Instruction::POP);
            }
            Stmt::Block(stmts) => {
                self.begin_scope();
                for stmt in stmts {
                    self.compile_stmt(stmt)?;
                }
                self.end_scope();
            }
            Stmt::If(cond, then_stmt, else_stmt) => {
                self.compile_expr(cond)?;
                let jump_to_else = self.builder.emit_jump(Instruction::JUMP_IF_FALSE);
                self.compile_stmt(then_stmt)?;
                let jump_over_else = self.builder.emit_jump(Instruction::JUMP);
                self.builder.patch_jump(jump_to_else);
                self.compile_stmt(else_stmt)?;
                self.builder.patch_jump(jump_over_else);
            }
            Stmt::Let(bindings) => {
                for (symbol, init) in bindings {
                    match init {
                        Some(expr) => self.compile_expr(expr)?,
                        None => self.builder.emit(Instruction::NULL),
                    }
                    if self.at_global() {
                        let idx = self.define_global(*symbol)?;
                        self.builder.emit(Instruction::DEFINE_GLOBAL);
                        self.builder.emit(idx);
                    } else {
                        self.add_local(*symbol);
                        // no instruction — value already sits in this local's slot
                    }
                }
            }
            Stmt::Assign(symbol, expr) => {
                self.compile_expr(expr)?;
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
                    VarLoc::Undefined => {
                        return Err(self.err(format!(
                            "assignment to undefined variable '{}'",
                            self.name_of(*symbol)
                        )));
                    }
                }
                self.builder.emit(Instruction::POP);
            }
            Stmt::Function(symbol, params, stmts) => {
                let name = self.name_of(*symbol);
                if self.at_global() {
                    let idx = self.define_global(*symbol)?;
                    self.compile_function(&name, params, stmts)?;
                    self.builder.emit(Instruction::DEFINE_GLOBAL);
                    self.builder.emit(idx);
                } else {
                    self.add_local(*symbol);
                    self.compile_function(&name, params, stmts)?;
                }
            }
            Stmt::Class(name, parent, body) => self.compile_class(name, *parent, body)?,
            Stmt::PropertyAssign(obj_expr, prop, value_expr) => {
                self.compile_expr(obj_expr)?;
                self.compile_expr(value_expr)?;
                let name_const = self.sym_const(*prop)?;
                self.builder.emit(Instruction::SET_PROPERTY);
                self.builder.emit(name_const);
                self.builder.emit(Instruction::POP);
            }
            Stmt::While(cond, body) => {
                // loop_start:
                //   <cond> ; JUMP_IF_FALSE exit ; <body> ; LOOP loop_start
                // exit:
                let loop_start = self.builder.here();
                self.compile_expr(cond)?;
                let exit_jump = self.builder.emit_jump(Instruction::JUMP_IF_FALSE);

                let depth = self.scope().scope_depth;
                self.loop_scopes.push(LoopScope {
                    break_jumps: Vec::new(),
                    continue_jumps: Vec::new(),
                    break_depth: depth,
                    continue_depth: depth,
                });

                self.compile_stmt(body)?;

                let loop_scope = self.loop_scopes.pop().unwrap();
                for c in loop_scope.continue_jumps {
                    self.builder.patch_jump(c);
                }
                self.builder.emit_loop(loop_start);
                self.builder.patch_jump(exit_jump);
                for b in loop_scope.break_jumps {
                    self.builder.patch_jump(b);
                }
            }
            Stmt::For(var, iterable, body) => self.compile_for(var, iterable, body)?,
            Stmt::Return(expr) => {
                if self.fn_scopes.len() == 1 {
                    return Err(self.err("'return' outside a function"));
                }
                self.compile_expr(expr)?;
                self.builder.emit(Instruction::RETURN);
            }
            Stmt::Break => {
                let Some(depth) = self.loop_scopes.last().map(|l| l.break_depth) else {
                    return Err(self.err("'break' outside a loop"));
                };
                self.emit_pops_to_depth(depth);
                let jump = self.builder.emit_jump(Instruction::JUMP);
                self.loop_scopes.last_mut().unwrap().break_jumps.push(jump);
            }
            Stmt::Continue => {
                let Some(depth) = self.loop_scopes.last().map(|l| l.continue_depth) else {
                    return Err(self.err("'continue' outside a loop"));
                };
                self.emit_pops_to_depth(depth);
                let jump = self.builder.emit_jump(Instruction::JUMP);
                self.loop_scopes
                    .last_mut()
                    .unwrap()
                    .continue_jumps
                    .push(jump);
            }
            Stmt::Import(package, imports) => self.compile_import(*package, imports)?,
        }
        Ok(())
    }

    fn compile_import(&mut self, package: Symbol, imports: &[Symbol]) -> Result<(), CompileError> {
        if !self.at_global() {
            return Err(self.err("imports are only allowed at the top level"));
        }

        let module = self.name_of(package);
        self.splice_module(package, &module)?;

        for name in imports {
            let src = self
                .globals
                .resolve(self.qualified(&module, *name))
                .ok_or_else(|| {
                    self.err(format!(
                        "module '{}' has no export '{}'",
                        module,
                        self.name_of(*name)
                    ))
                })?;
            let dest = self.define_global(*name)?;

            self.builder.emit(Instruction::GET_GLOBAL);
            self.builder.emit(src);
            self.builder.emit(Instruction::DEFINE_GLOBAL);
            self.builder.emit(dest);
        }
        Ok(())
    }

    fn splice_module(&mut self, package: Symbol, module: &str) -> Result<(), CompileError> {
        if self.loaded.contains(&package) {
            return Ok(());
        }
        if self.loading.contains(&package) {
            let mut chain: Vec<String> = self.loading.iter().map(|&m| self.name_of(m)).collect();
            chain.push(module.to_string());
            return Err(self.err(format!("circular import: {}", chain.join(" -> "))));
        }

        let source = self
            .loader
            .load(module)
            .map_err(|e| self.err(format!("cannot import '{}': {}", module, e)))?;

        let program = Parser::new(&source, self.ctx)
            .parse()
            .map_err(|e| self.err(format!("in module '{}': parse error: {}", module, e)))?;

        self.loading.push(package);
        let outer = self.module_prefix.replace(module.to_string());

        let mut result = Ok(());
        for stmt in &program.stmts {
            result = self.compile_stmt(stmt);
            if result.is_err() {
                break;
            }
        }

        self.module_prefix = outer;
        self.loading.pop();

        result.map_err(|e| CompileError {
            message: format!("in module '{}': {}", module, e),
            line: e.line,
        })?;

        self.loaded.insert(package);
        Ok(())
    }

    /// Compile `for var in iterable { body }` by desugaring to an index loop
    /// over the (list) iterable, using three hidden locals: the list, the
    /// index, and the loop variable. Wrapped in its own scope so the loop
    /// variables are locals even at top level.
    fn compile_for(
        &mut self,
        var: &Symbol,
        iterable: &Expr,
        body: &Stmt,
    ) -> Result<(), CompileError> {
        // `break` unwinds to the depth *outside* the loop, so it drops the three
        // hidden iteration locals along with the body's. `continue` stops one
        // level shallower — those three carry state into the next iteration.
        let break_depth = self.scope().scope_depth;
        self.begin_scope();
        let continue_depth = self.scope().scope_depth;

        // Unique names so nested `for` loops don't collide in the flat table.
        let uid = self.synthetic_counter;
        self.synthetic_counter += 1;
        let list_name = self.ctx.intern(&format!("$for_list{}", uid));
        let idx_name = self.ctx.intern(&format!("$for_idx{}", uid));

        // hidden: __list = iterable  (value stays in this local's slot)
        self.compile_expr(iterable)?;
        let list_slot = self.add_local(list_name);

        // hidden: __idx = 0
        self.builder
            .try_emit_constant(Constant::Int(0))
            .map_err(|e| self.err(e))?;
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

        self.loop_scopes.push(LoopScope {
            break_jumps: Vec::new(),
            continue_jumps: Vec::new(),
            break_depth,
            continue_depth,
        });

        self.compile_stmt(body)?;

        let loop_scope = self.loop_scopes.pop().unwrap();

        // `continue` lands on the increment, never on `loop_start` — skipping
        // the increment would spin forever.
        for c in loop_scope.continue_jumps {
            self.builder.patch_jump(c);
        }

        // idx = idx + 1
        self.builder.emit(Instruction::GET_LOCAL);
        self.builder.emit(idx_slot);
        self.builder
            .try_emit_constant(Constant::Int(1))
            .map_err(|e| self.err(e))?;
        self.builder.emit(Instruction::ADD);
        self.builder.emit(Instruction::SET_LOCAL);
        self.builder.emit(idx_slot);
        self.builder.emit(Instruction::POP);

        self.builder.emit_loop(loop_start);
        self.builder.patch_jump(exit_jump);

        // Discard the three hidden locals (var, idx, list).
        self.end_scope();

        // `break` already emitted its own pops for those three, so it has to
        // land *past* the ones `end_scope` just emitted for the normal exit.
        for b in loop_scope.break_jumps {
            self.builder.patch_jump(b);
        }
        Ok(())
    }

    fn compile_function(
        &mut self,
        name: &str,
        params: &ParamVec,
        body: &Stmt,
    ) -> Result<(), CompileError> {
        let jump_over = self.builder.emit_jump(Instruction::JUMP);
        let entry = self.builder.here();
        self.builder.name_fn(entry, name.to_string());

        self.fn_scopes.push(FnScope::new());

        // To stop break from escaping whole function
        let enclosing_loops = std::mem::take(&mut self.loop_scopes);

        for param in params {
            self.add_local(*param);
        }

        let body_result = self.compile_function_body(body);
        self.loop_scopes = enclosing_loops;
        body_result?;

        let scope = self.fn_scopes.pop().unwrap();

        self.builder.patch_jump(jump_over);

        let arity = params.len() as u8;
        if scope.upvalues.is_empty() {
            // Non-capturing: a flat function value, no heap allocation.
            self.builder
                .try_emit_constant(Constant::Fn { entry, arity })
                .map_err(|e| self.err(e))?;
        } else {
            // Capturing: emit CLOSURE with the capture descriptors.
            let fn_const = self
                .builder
                .try_add_constant(Constant::Fn { entry, arity })
                .map_err(|e| self.err(e))?;
            self.builder.emit(Instruction::CLOSURE);
            self.builder.emit(fn_const);
            self.builder.emit(scope.upvalues.len() as u8);
            for uv in &scope.upvalues {
                self.builder.emit(uv.is_local as u8);
                self.builder.emit(uv.index);
            }
        }
        Ok(())
    }

    fn compile_function_body(&mut self, body: &Stmt) -> Result<(), CompileError> {
        if let Stmt::Block(stmts) = body {
            self.scope_mut().scope_depth += 1;
            for stmt in stmts {
                self.compile_stmt(stmt)?;
            }
            self.discard_scope_locals();
        } else {
            self.compile_stmt(body)?;
        }

        self.builder.emit(Instruction::NULL);
        self.builder.emit(Instruction::RETURN);
        Ok(())
    }

    fn compile_class(
        &mut self,
        name: &Symbol,
        parent: Option<Symbol>,
        body: &[Stmt],
    ) -> Result<(), CompileError> {
        if !self.at_global() {
            return Err(self.err("classes can only be declared at top level"));
        }

        let class_idx = self.define_global(*name)?;

        let name_const = self.sym_const(*name)?;
        self.builder.emit(Instruction::CLASS);
        self.builder.emit(name_const);

        if let Some(parent) = parent {
            let idx = self.resolve_global(parent).ok_or_else(|| {
                self.err(format!("undefined parent class '{}'", self.name_of(parent)))
            })?;
            self.builder.emit(Instruction::GET_GLOBAL);
            self.builder.emit(idx);
            self.builder.emit(Instruction::INHERIT);
        }

        for member in body {
            match member {
                Stmt::Let(bindings) => {
                    for (sym, init) in bindings {
                        match init {
                            Some(expr) => self.compile_expr(expr)?,
                            None => self.builder.emit(Instruction::NULL),
                        }
                        let c = self.sym_const(*sym)?;
                        self.builder.emit(Instruction::STATIC_FIELD);
                        self.builder.emit(c);
                    }
                }
                Stmt::Function(fn_name, params, fn_body) => {
                    let method_name = format!("{}.{}", self.name_of(*name), self.name_of(*fn_name));
                    self.compile_function(&method_name, params, fn_body)?;
                    let c = self.sym_const(*fn_name)?;
                    self.builder.emit(Instruction::METHOD);
                    self.builder.emit(c);
                }
                _ => {}
            }
        }

        self.builder.emit(Instruction::DEFINE_GLOBAL);
        self.builder.emit(class_idx);
        Ok(())
    }

    pub fn compile_expr_only(mut self, expr: &Expr) -> Result<Bytecode, CompileError> {
        self.compile_expr(expr)?;
        self.builder.emit(Instruction::HALT);
        Ok(self.builder.build())
    }

    /// Like `compile`, but if the program ends in an expression statement its
    /// value is left on the stack instead of popped, so the caller (REPL,
    /// tests) can observe the result of the final expression.
    pub fn compile_repl(mut self, program: &Program) -> Result<Bytecode, CompileError> {
        if let Some((last, rest)) = program.stmts.split_last() {
            for stmt in rest {
                self.compile_stmt(stmt)?;
            }
            match last {
                Stmt::Expr(expr) => self.compile_expr(expr)?,
                other => self.compile_stmt(other)?,
            }
        }
        self.builder.emit(Instruction::HALT);
        Ok(self.builder.build())
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), CompileError> {
        self.mark_line(expr.line);
        let line = expr.line;
        match &expr.kind {
            ExprKind::Literal(lit) => self.compile_literal(lit)?,
            ExprKind::List(elements) => {
                for element in elements {
                    self.compile_expr(element)?;
                }
                self.mark_line(line);
                self.builder.emit(Instruction::BUILD_LIST);
                self.builder.emit(elements.len() as u8);
            }
            ExprKind::Binary(op, lhs, rhs) => self.compile_binary(op, lhs, rhs, line)?,
            ExprKind::Unary(op, operand) => self.compile_unary(op, operand, line)?,
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
                VarLoc::Undefined => {
                    return Err(self.err(format!("undefined variable '{}'", self.name_of(*var))));
                }
            },
            ExprKind::Call(name, args) => {
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
                    VarLoc::Undefined => {
                        return Err(
                            self.err(format!("undefined function '{}'", self.name_of(*name)))
                        );
                    }
                }
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.mark_line(line);
                self.builder.emit(Instruction::CALL);
                self.builder.emit(args.len() as u8);
            }
            ExprKind::New(class, args) => {
                let idx = self.globals.resolve(*class).ok_or_else(|| {
                    self.err(format!("undefined class '{}'", self.name_of(*class)))
                })?;
                self.builder.emit(Instruction::GET_GLOBAL);
                self.builder.emit(idx);
                for arg in args {
                    self.compile_expr(arg)?;
                }
                let init_const = self.sym_const(self.ctx.intern("init"))?;
                self.mark_line(line);
                self.builder.emit(Instruction::NEW);
                self.builder.emit(init_const);
                self.builder.emit(args.len() as u8);
            }
            ExprKind::Property(obj, name) => {
                self.compile_expr(obj)?;
                let c = self.sym_const(*name)?;
                self.mark_line(line);
                self.builder.emit(Instruction::GET_PROPERTY);
                self.builder.emit(c);
            }
            ExprKind::MethodCall(obj, method, args) => {
                self.compile_expr(obj)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                let c = self.sym_const(*method)?;
                self.mark_line(line);
                self.builder.emit(Instruction::INVOKE);
                self.builder.emit(c);
                self.builder.emit(args.len() as u8);
            }
            ExprKind::StaticProperty(obj, name) => {
                self.compile_expr(obj)?;
                let c = self.sym_const(*name)?;
                self.mark_line(line);
                self.builder.emit(Instruction::GET_STATIC);
                self.builder.emit(c);
            }
            ExprKind::StaticMethodCall(obj, method, args) => {
                self.compile_expr(obj)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                let c = self.sym_const(*method)?;
                self.mark_line(line);
                self.builder.emit(Instruction::STATIC_INVOKE);
                self.builder.emit(c);
                self.builder.emit(args.len() as u8);
            }
            ExprKind::Lambda(..) => {
                return Err(self.err("lambda expressions are not supported by the VM yet"));
            }
        }
        Ok(())
    }

    fn compile_literal(&mut self, lit: &Literal) -> Result<(), CompileError> {
        match lit {
            Literal::Null => self.builder.emit(Instruction::NULL),
            Literal::Bool(true) => self.builder.emit(Instruction::TRUE),
            Literal::Bool(false) => self.builder.emit(Instruction::FALSE),
            Literal::Int(n) => self
                .builder
                .try_emit_constant(Constant::Int(*n))
                .map_err(|e| self.err(e))?,
            Literal::Float(n) => self
                .builder
                .try_emit_constant(Constant::Float(*n))
                .map_err(|e| self.err(e))?,
            Literal::Str(s) => {
                let string = self.ctx.resolve(*s);
                self.builder
                    .try_emit_constant(Constant::Str(string))
                    .map_err(|e| self.err(e))?
            }
        }
        Ok(())
    }

    fn compile_binary(
        &mut self,
        op: &Operation,
        lhs: &Expr,
        rhs: &Expr,
        line: u32,
    ) -> Result<(), CompileError> {
        if let (Some(a), Some(b)) = (fold_const(lhs), fold_const(rhs))
            && let Some(folded) = fold_binary(op, a, b)
        {
            return self.compile_literal(&folded);
        }

        self.compile_expr(lhs)?;
        self.compile_expr(rhs)?;
        self.mark_line(line);

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
        Ok(())
    }

    fn compile_unary(
        &mut self,
        op: &UnaryOp,
        operand: &Expr,
        line: u32,
    ) -> Result<(), CompileError> {
        if let Some(v) = fold_const(operand)
            && let Some(folded) = fold_unary(op, v)
        {
            return self.compile_literal(&folded);
        }

        self.compile_expr(operand)?;
        self.mark_line(line);

        let instruction = match op {
            UnaryOp::Neg => Instruction::NEG,
            UnaryOp::Not => Instruction::NOT,
            UnaryOp::Inv => Instruction::BITINV,
        };
        self.builder.emit(instruction);
        Ok(())
    }
}

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
        (Add, Int(x), Int(y)) => x.checked_add(y).map(Int),
        (Sub, Int(x), Int(y)) => x.checked_sub(y).map(Int),
        (Mul, Int(x), Int(y)) => x.checked_mul(y).map(Int),
        (Div, Int(x), Int(y)) => x.checked_div(y).map(Int),
        (Mod, Int(x), Int(y)) => x.checked_rem(y).map(Int),

        (Add, Float(x), Float(y)) => Some(Float(x + y)),
        (Sub, Float(x), Float(y)) => Some(Float(x - y)),
        (Mul, Float(x), Float(y)) => Some(Float(x * y)),
        (Div, Float(x), Float(y)) => Some(Float(x / y)),
        (Mod, Float(x), Float(y)) => Some(Float(x % y)),

        (Gt, Int(x), Int(y)) => Some(Bool(x > y)),
        (Lt, Int(x), Int(y)) => Some(Bool(x < y)),
        (Gte, Int(x), Int(y)) => Some(Bool(x >= y)),
        (Lte, Int(x), Int(y)) => Some(Bool(x <= y)),
        (Gt, Float(x), Float(y)) => Some(Bool(x > y)),
        (Lt, Float(x), Float(y)) => Some(Bool(x < y)),
        (Gte, Float(x), Float(y)) => Some(Bool(x >= y)),
        (Lte, Float(x), Float(y)) => Some(Bool(x <= y)),

        (Eq, Int(x), Int(y)) => Some(Bool(x == y)),
        (Neq, Int(x), Int(y)) => Some(Bool(x != y)),
        (Eq, Float(x), Float(y)) => Some(Bool(x == y)),
        (Neq, Float(x), Float(y)) => Some(Bool(x != y)),
        (Eq, Bool(x), Bool(y)) => Some(Bool(x == y)),
        (Neq, Bool(x), Bool(y)) => Some(Bool(x != y)),

        (And, Bool(x), Bool(y)) => Some(Bool(x && y)),
        (Or, Bool(x), Bool(y)) => Some(Bool(x || y)),

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
        let bytecode = compiler.compile_expr_only(&expr).expect("compile failed");
        let mut vm = AxeVM::new(&bytecode);
        vm.exec().expect("runtime error")
    }

    /// Run an expression and render its result via the VM's heap. Needed for
    /// string results, whose `Value`s are opaque `ObjRef` handles.
    fn compile_and_display(ctx: &Context, expr: Expr) -> Option<String> {
        let compiler = Compiler::new(ctx);
        let bytecode = compiler.compile_expr_only(&expr).expect("compile failed");
        let mut vm = AxeVM::new(&bytecode);
        let result = vm.exec().expect("runtime error");
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

        let bytecode = Compiler::new(&ctx).compile_expr_only(&expr).unwrap();

        // Whole tree collapses: one constant, and code is just CONST 0; HALT.
        assert_eq!(bytecode.constants, vec![Constant::Int(55)]);
        assert_eq!(
            bytecode.code,
            vec![Instruction::CONST, 0, Instruction::HALT]
        );

        // ...and it still evaluates correctly.
        let mut vm = AxeVM::new(&bytecode);
        assert_eq!(vm.exec().unwrap(), Some(Value::Int(55)));
    }

    #[test]
    fn test_constant_folding_skips_div_by_zero() {
        let ctx = Context::new();

        // 1 / 0 must NOT be folded — the DIV opcode stays so the VM reports
        // a runtime error exactly as it would without folding.
        let expr = Expr::Binary(
            Operation::Div,
            Box::new(Expr::Literal(Literal::Int(1))),
            Box::new(Expr::Literal(Literal::Int(0))),
        );

        let bytecode = Compiler::new(&ctx).compile_expr_only(&expr).unwrap();
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
            compiler.compile_stmt(stmt).expect("compile failed");
        }
        match last {
            Stmt::Expr(expr) => compiler.compile_expr(expr).expect("compile failed"),
            other => compiler.compile_stmt(other).expect("compile failed"),
        }
        compiler.builder.emit(Instruction::HALT);
        let bytecode = compiler.builder.build();

        let mut vm = AxeVM::new(&bytecode);
        vm.exec()
            .expect("runtime error")
            .map(|v| vm.display_value(&v))
    }

    /// Compile a whole program expecting it to be *rejected*, returning the
    /// error message. Panics if it compiles.
    fn compile_error(src: &str) -> String {
        let ctx = Context::new();
        let program = crate::parser::Parser::new(src, &ctx)
            .parse()
            .expect("parse failed");
        match Compiler::new(&ctx).compile(&program) {
            Ok(_) => panic!("expected a compile error, but it compiled"),
            Err(e) => e.message,
        }
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

    // ---- break / continue ----------------------------------------------
    //
    // These loops jump out of a scope whose `end_scope` they never reach, so
    // the compiler has to emit the matching POP/CLOSE_UPVALUE run inline at
    // the jump site. Most of the assertions below are really stack-balance
    // assertions in disguise: a missing or duplicated pop shifts every later
    // local slot, so the observable symptom is a variable reading as the
    // wrong value rather than a crash.

    #[test]
    fn test_break_in_while() {
        let out = run_source(
            "let i = 0; let total = 0;
             while (i < 10) {
                 if (i == 3) { break; }
                 total = total + i;
                 i = i + 1;
             }
             total;",
        );
        assert_eq!(out, Some("3".to_string())); // 0+1+2
    }

    #[test]
    fn test_continue_in_while() {
        let out = run_source(
            "let j = 0; let sum = 0;
             while (j < 6) {
                 j = j + 1;
                 if (j == 2) { continue; }
                 sum = sum + j * 10;
             }
             sum;",
        );
        assert_eq!(out, Some("190".to_string())); // 10+30+40+50+60
    }

    // The stack-balance tests below all share a shape, and it is load-bearing:
    // everything sits inside a `fn` (at top level `let` makes globals, which
    // live outside the value stack entirely) and a local is declared *after*
    // the loop (a leak strands values above the live locals, shifting nothing
    // until a later slot is assigned — and `RETURN` truncates the frame, so a
    // leak immediately before a return is invisible too). With `after` present,
    // a missing pop means the compiler calls it slot N while the runtime pushed
    // it to N+k, and the read comes back as the leaked value.

    #[test]
    fn test_break_pops_body_locals() {
        let out = run_source(
            "fn f() {
                 let i = 0;
                 while (i < 10) {
                     let a = 100;
                     let b = 200;
                     if (i == 3) { break; }
                     i = i + 1;
                 }
                 let after = 7;
                 return after;
             }
             f();",
        );
        assert_eq!(out, Some("7".to_string())); // 100 if `a`/`b` leaked
    }

    #[test]
    fn test_continue_pops_body_locals() {
        // A leaking `continue` strands a fresh pair every iteration, so this
        // also catches unbounded stack growth.
        let out = run_source(
            "fn f() {
                 let i = 0; let sum = 0;
                 while (i < 5) {
                     let a = 1;
                     let b = 2;
                     i = i + 1;
                     if (i == 2) { continue; }
                     sum = sum + i;
                 }
                 let after = 7;
                 return after;
             }
             f();",
        );
        assert_eq!(out, Some("7".to_string()));
    }

    #[test]
    fn test_break_unwinds_multiple_scopes() {
        // The break sits two blocks deep inside the loop body, so it has to
        // unwind both levels at once — more than any single `end_scope` does.
        let out = run_source(
            "fn f() {
                 let q = 0;
                 while (q < 9) {
                     let a = 1;
                     if (q > 0) { let b = 2; if (b == 2) { break; } }
                     q = q + 1;
                 }
                 let after = 7;
                 return q * 100 + after;
             }
             f();",
        );
        assert_eq!(out, Some("107".to_string())); // q == 1, after == 7
    }

    #[test]
    fn test_break_binds_to_innermost_loop() {
        let out = run_source(
            "let o = 0; let hits = 0;
             while (o < 3) {
                 let p = 0;
                 while (p < 10) {
                     if (p == 2) { break; }
                     p = p + 1;
                 }
                 hits = hits + p;
                 o = o + 1;
             }
             hits;",
        );
        assert_eq!(out, Some("6".to_string())); // 2+2+2, outer loop unaffected
    }

    #[test]
    fn test_break_in_for() {
        let out = run_source(
            "let acc = 0;
             for i in range(0, 10) {
                 let junk = 999;
                 if (i == 4) { break; }
                 acc = acc + i;
             }
             acc;",
        );
        assert_eq!(out, Some("6".to_string())); // 0+1+2+3
    }

    #[test]
    fn test_break_in_for_pops_hidden_locals() {
        // `for` keeps three hidden locals (list, index, loop var) plus the
        // body's. `break` drops all of them itself, and must then land *past*
        // the pops `end_scope` emits for the normal exit or the three get
        // popped twice — which underflows into whatever sits below the loop.
        let out = run_source(
            "fn f() {
                 let acc = 0;
                 for i in range(0, 10) {
                     let junk = 999;
                     if (i == 4) { break; }
                     acc = acc + i;
                 }
                 let after = 7;
                 return acc * 100 + after;
             }
             f();",
        );
        assert_eq!(out, Some("607".to_string())); // acc == 6, after == 7
    }

    #[test]
    fn test_continue_in_for_still_advances() {
        // `continue` must jump to the increment, not to the condition, and must
        // *keep* the three hidden locals alive for the next iteration.
        let out = run_source(
            "let ev = 0;
             for i in range(0, 6) {
                 if (i == 2) { continue; }
                 let w = i;
                 ev = ev + w;
             }
             ev;",
        );
        assert_eq!(out, Some("13".to_string())); // 0+1+3+4+5
    }

    #[test]
    fn test_break_in_for_nested_in_while() {
        let out = run_source(
            "let outer = 0; let tot = 0;
             while (outer < 2) {
                 for i in range(0, 5) {
                     if (i == 3) { break; }
                     tot = tot + 1;
                 }
                 outer = outer + 1;
             }
             tot;",
        );
        assert_eq!(out, Some("6".to_string())); // 3+3
    }

    #[test]
    fn test_locals_before_loop_survive_break() {
        // `before` and the param `n` sit below the loop in the same frame and
        // must not be popped — over-popping is as wrong as under-popping.
        let out = run_source(
            "fn check(n) {
                 let before = 42;
                 for i in range(0, n) {
                     let t = i;
                     if (t == 1) { break; }
                 }
                 let after = 7;
                 return before * 100 + n * 10 + after;
             }
             check(5);",
        );
        assert_eq!(out, Some("4257".to_string())); // before 42, n 5, after 7
    }

    #[test]
    fn test_continue_in_for_keeps_hidden_locals() {
        // The mirror of the break case: `continue` must stop one level short of
        // the three hidden locals, which carry the iteration state forward. Pop
        // them and the loop loses its index.
        let out = run_source(
            "fn f() {
                 let seen = 0;
                 for i in range(0, 6) {
                     let w = i;
                     if (w == 2) { continue; }
                     seen = seen + w;
                 }
                 let after = 7;
                 return seen * 100 + after;
             }
             f();",
        );
        assert_eq!(out, Some("1307".to_string())); // seen 13, after 7
    }

    #[test]
    fn test_break_closes_captured_local() {
        // `boxed` is captured, so `break` must emit CLOSE_UPVALUE, not POP.
        // `after` deliberately reuses the stack slot `boxed` occupied: with a
        // plain POP the upvalue would still point at that slot and the closure
        // would return 7 instead of 22.
        let out = run_source(
            "fn make() {
                 let saved = null;
                 let k = 0;
                 while (k < 5) {
                     let boxed = k * 11;
                     fn get() { return boxed; }
                     saved = get;
                     k = k + 1;
                     if (k == 3) { break; }
                 }
                 let after = 7;
                 return saved();
             }
             make();",
        );
        assert_eq!(out, Some("22".to_string()));
    }

    #[test]
    fn test_continue_closes_captured_local() {
        let out = run_source(
            "fn make() {
                 let saved = null;
                 for i in range(0, 4) {
                     let v = i + 100;
                     fn peek() { return v; }
                     saved = peek;
                     if (i == 1) { continue; }
                 }
                 let after = 9;
                 return saved();
             }
             make();",
        );
        assert_eq!(out, Some("103".to_string()));
    }

    #[test]
    fn test_dead_code_after_break_still_compiles() {
        // The compiler is single-pass with no reachability analysis, so `dead`
        // is compiled and allocated a slot like any other local. It must not
        // disturb the locals declared around it.
        let out = run_source(
            "let n = 0;
             while (n < 5) {
                 let live = 1;
                 if (n == 2) { break; let dead = 99; }
                 n = n + live;
             }
             n;",
        );
        assert_eq!(out, Some("2".to_string()));
    }

    #[test]
    fn test_break_outside_loop_is_an_error() {
        assert_eq!(compile_error("break;"), "'break' outside a loop");
        assert_eq!(
            compile_error("let x = 1; continue;"),
            "'continue' outside a loop"
        );
    }

    #[test]
    fn test_break_does_not_escape_a_function_body() {
        // A function body is a different call frame, so an enclosing loop's exit
        // is not a legal jump target from inside it. Without the boundary this
        // silently emitted a JUMP patched into the caller's code.
        assert_eq!(
            compile_error(
                "let k = 0;
                 while (k < 3) {
                     fn inner() { break; }
                     inner();
                     k = k + 1;
                 }"
            ),
            "'break' outside a loop"
        );
        assert_eq!(
            compile_error("for i in range(0, 3) { fn f() { continue; } }"),
            "'continue' outside a loop"
        );
    }

    #[test]
    fn test_loop_after_function_still_allows_break() {
        // The mem::take in compile_function must *restore* the enclosing loops,
        // not drop them: a break after a nested fn declaration is still legal.
        let out = run_source(
            "let c = 0;
             while (c < 10) {
                 fn noop() { return 1; }
                 c = c + noop();
                 if (c == 4) { break; }
             }
             c;",
        );
        assert_eq!(out, Some("4".to_string()));
    }

    // ---- modules / imports ----

    /// A `ModuleLoader` over an in-memory name -> source map, so module tests
    /// don't touch the filesystem.
    struct MapLoader(std::collections::HashMap<String, String>);

    impl ModuleLoader for MapLoader {
        fn load(&self, name: &str) -> Result<String, String> {
            self.0
                .get(name)
                .cloned()
                .ok_or_else(|| format!("no such module '{}'", name))
        }
    }

    fn map_loader(modules: &[(&str, &str)]) -> Box<dyn ModuleLoader> {
        Box::new(MapLoader(
            modules
                .iter()
                .map(|(n, s)| (n.to_string(), s.to_string()))
                .collect(),
        ))
    }

    /// `run_source`, but with a set of importable modules.
    fn run_with_modules(src: &str, modules: &[(&str, &str)]) -> Option<String> {
        let ctx = Context::new();
        let program = crate::parser::Parser::new(src, &ctx)
            .parse()
            .expect("parse failed");

        let mut compiler = Compiler::with_loader(&ctx, map_loader(modules));
        let (last, rest) = program.stmts.split_last().expect("empty program");
        for stmt in rest {
            compiler.compile_stmt(stmt).expect("compile failed");
        }
        match last {
            Stmt::Expr(expr) => compiler.compile_expr(expr).expect("compile failed"),
            other => compiler.compile_stmt(other).expect("compile failed"),
        }
        compiler.builder.emit(Instruction::HALT);
        let bytecode = compiler.builder.build();

        let mut vm = AxeVM::new(&bytecode);
        vm.exec()
            .expect("runtime error")
            .map(|v| vm.display_value(&v))
    }

    /// `compile_error`, but with a set of importable modules.
    fn module_compile_error(src: &str, modules: &[(&str, &str)]) -> String {
        let ctx = Context::new();
        let program = crate::parser::Parser::new(src, &ctx)
            .parse()
            .expect("parse failed");
        match Compiler::with_loader(&ctx, map_loader(modules)).compile(&program) {
            Ok(_) => panic!("expected a compile error, but it compiled"),
            Err(e) => e.message,
        }
    }

    #[test]
    fn test_import_binds_module_function() {
        assert_eq!(
            run_with_modules(
                "from math import add;
                 add(1, 2);",
                &[("math", "fn add(a, b) { return a + b; }")],
            ),
            Some("3".to_string())
        );
    }

    #[test]
    fn test_import_binds_module_class() {
        assert_eq!(
            run_with_modules(
                "from shapes import Square;
                 let s = new Square(4);
                 s.area();",
                &[(
                    "shapes",
                    "class Square {
                         fn init(self, side) { self.side = side; }
                         fn area(self) { return self.side * self.side; }
                     }"
                )],
            ),
            Some("16".to_string())
        );
    }

    #[test]
    fn test_module_body_runs_once() {
        // `n` is incremented by the module's top-level code. Importing twice
        // must not splice — and so must not re-run — that body.
        assert_eq!(
            run_with_modules(
                "from counter import n;
                 from counter import n;
                 n;",
                &[("counter", "let n = 0; n = n + 1;")],
            ),
            Some("1".to_string())
        );
    }

    #[test]
    fn test_diamond_import_runs_body_once() {
        // main -> a -> base, main -> base. `base` is spliced exactly once.
        assert_eq!(
            run_with_modules(
                "from a import bump;
                 from base import n;
                 bump();
                 n;",
                &[
                    ("base", "let n = 0; n = n + 1;"),
                    (
                        "a",
                        "from base import n;
                         fn bump() { return n; }"
                    ),
                ],
            ),
            Some("1".to_string())
        );
    }

    #[test]
    fn test_module_imports_module() {
        assert_eq!(
            run_with_modules(
                "from util import twice;
                 twice(21);",
                &[
                    ("math", "fn add(a, b) { return a + b; }"),
                    (
                        "util",
                        "from math import add;
                         fn twice(x) { return add(x, x); }"
                    ),
                ],
            ),
            Some("42".to_string())
        );
    }

    #[test]
    fn test_module_private_name_is_not_imported() {
        // `helper` is defined by the module but not imported, so it is not in
        // the importer's namespace.
        let err = module_compile_error(
            "from math import add;
             helper(1);",
            &[(
                "math",
                "fn helper(x) { return x; }
                 fn add(a, b) { return a + b; }",
            )],
        );
        assert!(err.contains("undefined function 'helper'"), "{}", err);
    }

    #[test]
    fn test_module_and_importer_can_share_a_name() {
        // Both define `helper`; the module's calls must reach its own.
        assert_eq!(
            run_with_modules(
                "from math import triple;
                 fn helper(x) { return x + 1000; }
                 triple(2) + helper(0);",
                &[(
                    "math",
                    "fn helper(x) { return x * 3; }
                     fn triple(x) { return helper(x); }"
                )],
            ),
            Some("1006".to_string())
        );
    }

    #[test]
    fn test_module_can_use_builtins() {
        assert_eq!(
            run_with_modules(
                "from util import size;
                 size([1, 2, 3]);",
                &[("util", "fn size(l) { return len(l); }")],
            ),
            Some("3".to_string())
        );
    }

    #[test]
    fn test_unknown_export_errors() {
        let err = module_compile_error(
            "from math import subtract;",
            &[("math", "fn add(a, b) { return a + b; }")],
        );
        assert_eq!(err, "module 'math' has no export 'subtract'");
    }

    #[test]
    fn test_missing_module_errors() {
        let err = module_compile_error("from nope import thing;", &[]);
        assert!(err.starts_with("cannot import 'nope'"), "{}", err);
    }

    #[test]
    fn test_circular_import_errors() {
        let err = module_compile_error(
            "from a import f;",
            &[
                ("a", "from b import g; fn f() { return g(); }"),
                ("b", "from a import f; fn g() { return 1; }"),
            ],
        );
        assert!(err.contains("circular import: a -> b -> a"), "{}", err);
    }

    #[test]
    fn test_import_inside_function_errors() {
        let err = module_compile_error(
            "fn f() { from math import add; }",
            &[("math", "fn add(a, b) { return a + b; }")],
        );
        assert_eq!(err, "imports are only allowed at the top level");
    }

    #[test]
    fn test_module_compile_error_names_the_module() {
        let err = module_compile_error("from bad import f;", &[("bad", "fn f() { return x; }")]);
        assert!(err.starts_with("in module 'bad':"), "{}", err);
        assert!(err.contains("undefined variable 'x'"), "{}", err);
    }
}
