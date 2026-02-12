use crate::ast::{Expr, Literal, Operation, Program, Stmt, UnaryOp};

use super::instructions::Instruction;
use super::vm::{Chunk, Value};

/// Compiles AST to bytecode
pub struct Compiler {
    chunk: Chunk,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            chunk: Chunk::new(),
        }
    }

    /// Compile a program and return the bytecode chunk
    pub fn compile(mut self, program: &Program) -> Chunk {
        for stmt in &program.stmts {
            self.compile_stmt(stmt);
        }
        self.chunk.emit(Instruction::HALT);
        self.chunk
    }

    /// Compile a single expression and return the bytecode chunk
    pub fn compile_expr_only(mut self, expr: &Expr) -> Chunk {
        self.compile_expr(expr);
        self.chunk.emit(Instruction::HALT);
        self.chunk
    }

    fn compile_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                self.compile_expr(expr);
                // Pop the result since it's an expression statement
                self.chunk.emit(Instruction::POP);
            }
            Stmt::Block(stmts) => {
                for stmt in stmts {
                    self.compile_stmt(stmt);
                }
            }
            // TODO: Implement other statements
            _ => todo!("Statement not yet implemented: {:?}", stmt),
        }
    }

    fn compile_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(lit) => self.compile_literal(lit),
            Expr::Binary(op, lhs, rhs) => self.compile_binary(op, lhs, rhs),
            Expr::Unary(op, operand) => self.compile_unary(op, operand),
            // TODO: Implement other expressions
            _ => todo!("Expression not yet implemented: {:?}", expr),
        }
    }

    fn compile_literal(&mut self, lit: &Literal) {
        match lit {
            Literal::Null => self.chunk.emit(Instruction::NULL),
            Literal::Bool(true) => self.chunk.emit(Instruction::TRUE),
            Literal::Bool(false) => self.chunk.emit(Instruction::FALSE),
            Literal::Int(n) => self.chunk.emit_constant(Value::Int(*n)),
            Literal::Float(n) => self.chunk.emit_constant(Value::Float(*n)),
            Literal::Str(s) => self.chunk.emit_constant(Value::Str(s.clone())),
        }
    }

    fn compile_binary(&mut self, op: &Operation, lhs: &Expr, rhs: &Expr) {
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
        self.chunk.emit(instruction);
    }

    fn compile_unary(&mut self, op: &UnaryOp, operand: &Expr) {
        self.compile_expr(operand);

        let instruction = match op {
            UnaryOp::Neg => Instruction::NEG,
            UnaryOp::Not => Instruction::NOT,
            UnaryOp::Inv => Instruction::BITINV,
        };
        self.chunk.emit(instruction);
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stack_vm::AxeVM;

    fn compile_and_run(expr: Expr) -> Option<Value> {
        let compiler = Compiler::new();
        let chunk = compiler.compile_expr_only(&expr);
        let mut vm = AxeVM::new(&chunk);
        vm.exec()
    }

    #[test]
    fn test_compile_literals() {
        // Null
        assert_eq!(
            compile_and_run(Expr::Literal(Literal::Null)),
            Some(Value::Null)
        );

        // Bool
        assert_eq!(
            compile_and_run(Expr::Literal(Literal::Bool(true))),
            Some(Value::Bool(true))
        );

        // Int
        assert_eq!(
            compile_and_run(Expr::Literal(Literal::Int(42))),
            Some(Value::Int(42))
        );

        // Float
        assert_eq!(
            compile_and_run(Expr::Literal(Literal::Float(3.14))),
            Some(Value::Float(3.14))
        );

        // String
        assert_eq!(
            compile_and_run(Expr::Literal(Literal::Str("hello".to_string()))),
            Some(Value::Str("hello".to_string()))
        );
    }

    #[test]
    fn test_compile_binary_arithmetic() {
        // 10 + 20
        let expr = Expr::Binary(
            Operation::Add,
            Box::new(Expr::Literal(Literal::Int(10))),
            Box::new(Expr::Literal(Literal::Int(20))),
        );
        assert_eq!(compile_and_run(expr), Some(Value::Int(30)));

        // 50 - 20
        let expr = Expr::Binary(
            Operation::Sub,
            Box::new(Expr::Literal(Literal::Int(50))),
            Box::new(Expr::Literal(Literal::Int(20))),
        );
        assert_eq!(compile_and_run(expr), Some(Value::Int(30)));

        // 6 * 7
        let expr = Expr::Binary(
            Operation::Mul,
            Box::new(Expr::Literal(Literal::Int(6))),
            Box::new(Expr::Literal(Literal::Int(7))),
        );
        assert_eq!(compile_and_run(expr), Some(Value::Int(42)));

        // 100 / 4
        let expr = Expr::Binary(
            Operation::Div,
            Box::new(Expr::Literal(Literal::Int(100))),
            Box::new(Expr::Literal(Literal::Int(4))),
        );
        assert_eq!(compile_and_run(expr), Some(Value::Int(25)));

        // 17 % 5
        let expr = Expr::Binary(
            Operation::Mod,
            Box::new(Expr::Literal(Literal::Int(17))),
            Box::new(Expr::Literal(Literal::Int(5))),
        );
        assert_eq!(compile_and_run(expr), Some(Value::Int(2)));
    }

    #[test]
    fn test_compile_binary_comparison() {
        // 5 > 3
        let expr = Expr::Binary(
            Operation::Gt,
            Box::new(Expr::Literal(Literal::Int(5))),
            Box::new(Expr::Literal(Literal::Int(3))),
        );
        assert_eq!(compile_and_run(expr), Some(Value::Bool(true)));

        // 5 < 3
        let expr = Expr::Binary(
            Operation::Lt,
            Box::new(Expr::Literal(Literal::Int(5))),
            Box::new(Expr::Literal(Literal::Int(3))),
        );
        assert_eq!(compile_and_run(expr), Some(Value::Bool(false)));

        // 5 == 5
        let expr = Expr::Binary(
            Operation::Eq,
            Box::new(Expr::Literal(Literal::Int(5))),
            Box::new(Expr::Literal(Literal::Int(5))),
        );
        assert_eq!(compile_and_run(expr), Some(Value::Bool(true)));

        // 5 != 3
        let expr = Expr::Binary(
            Operation::Neq,
            Box::new(Expr::Literal(Literal::Int(5))),
            Box::new(Expr::Literal(Literal::Int(3))),
        );
        assert_eq!(compile_and_run(expr), Some(Value::Bool(true)));
    }

    #[test]
    fn test_compile_unary() {
        // -42
        let expr = Expr::Unary(UnaryOp::Neg, Box::new(Expr::Literal(Literal::Int(42))));
        assert_eq!(compile_and_run(expr), Some(Value::Int(-42)));

        // !true
        let expr = Expr::Unary(UnaryOp::Not, Box::new(Expr::Literal(Literal::Bool(true))));
        assert_eq!(compile_and_run(expr), Some(Value::Bool(false)));
    }

    #[test]
    fn test_compile_complex_expression() {
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
        assert_eq!(compile_and_run(expr), Some(Value::Int(55)));
    }

    #[test]
    fn test_compile_string_concat() {
        // "Hello, " + "World!"
        let expr = Expr::Binary(
            Operation::Add,
            Box::new(Expr::Literal(Literal::Str("Hello, ".to_string()))),
            Box::new(Expr::Literal(Literal::Str("World!".to_string()))),
        );
        assert_eq!(
            compile_and_run(expr),
            Some(Value::Str("Hello, World!".to_string()))
        );
    }
}
