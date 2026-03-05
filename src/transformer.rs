use crate::ast::{Program, Stmt};

/// AST Transformer - currently a no-op pass.
///
/// All constructs are handled natively by the interpreter and resolver:
/// - `fn` (statement) and lambda (expression) - both create Value::Function
/// - `for` loops - handled directly in interpreter, not sugar for `while`
pub struct Transformer;

impl Transformer {
    pub fn transform_stmt(&self, stmt: Stmt) -> Stmt {
        stmt
    }

    pub fn transform_program(&self, program: Program) -> Program {
        Program {
            stmts: program
                .stmts
                .into_iter()
                .map(|s| self.transform_stmt(s))
                .collect(),
        }
    }
}
