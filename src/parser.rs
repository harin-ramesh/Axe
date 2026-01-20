use crate::tokeniser::{Token, TokenKind, Tokeniser};
use crate::{Condition, Expr, Operation};

pub struct Parser<'src> {
    tokeniser: Tokeniser<'src>,
    lookahead: Option<Token<'src>>,
}

impl<'src> Parser<'src> {
    pub fn new(input: &'src str) -> Self {
        let tokeniser = Tokeniser::new(input);
        Self {
            tokeniser,
            lookahead: None,
        }
    }

    pub fn parse(&mut self) -> Result<Expr, &'static str> {
        self.lookahead = Some(self.tokeniser.get_next_token()?);
        let expr = self.parse_block()?;

        Ok(expr)
    }

    fn eat(&mut self, expected_token: TokenKind) -> Result<Token<'src>, &'static str> {
        let token = match self.lookahead.take() {
            None => return Err("Unexpected end of input"),
            Some(t) => t,
        };

        if token.kind != expected_token {
            return Err("Unexpected token");
        }

        self.lookahead = Some(self.tokeniser.get_next_token()?);
        Ok(token)
    }

    fn parse_block(&mut self) -> Result<Expr, &'static str> {
        let stmts = self.parse_statements(TokenKind::Eof)?;
        Ok(Expr::Block(stmts))
    }

    // StatementList
    //  : Statement
    //  | StatemtnList Statement -> Statement Statement Statment
    fn parse_statements(&mut self, stop_token: TokenKind) -> Result<Vec<Expr>, &'static str> {
        let mut stmts = vec![self.parse_statement()?];

        while let Some(token) = &self.lookahead {
            if token.kind == stop_token {
                break;
            }
            stmts.push(self.parse_statement()?);
        }

        Ok(stmts)
    }

    // Statement
    //  : ExpressionStatement
    //  | BlockStatment
    //  | LetStatement
    //  | IfStatement
    fn parse_statement(&mut self) -> Result<Expr, &'static str> {
        let expr = match self.lookahead.map(|t| t.kind) {
            Some(TokenKind::OpeningBrace) => self.parse_block_statemnt()?,
            Some(TokenKind::Let) => self.parse_let_statement()?,
            Some(TokenKind::If) => self.parse_if_statement()?,
            _ => self.parse_expression_statemnt()?,
        };
        Ok(expr)
    }

    // LetStatement
    //  : 'let' DeclarationList ';'
    fn parse_let_statement(&mut self) -> Result<Expr, &'static str> {
        self.eat(TokenKind::Let)?;
        let declarations = self.parse_declarations()?;
        self.eat(TokenKind::Delimeter)?;
        Ok(Expr::Let(declarations))
    }

    // DeclarationList
    //  : Declaration
    //  | DeclarationList ',' Declaration
    fn parse_declarations(&mut self) -> Result<Vec<Expr>, &'static str> {
        let mut decls = vec![self.parse_declaration()?];

        while let Some(token) = &self.lookahead {
            if token.kind != TokenKind::Comma {
                break;
            }
            self.eat(TokenKind::Comma)?;
            decls.push(self.parse_declaration()?);
        }

        Ok(decls)
    }

    // Declaration
    //  : Identifier
    //  | Identifier '=' Expression
    fn parse_declaration(&mut self) -> Result<Expr, &'static str> {
        let name_token = self.eat(TokenKind::Identifier)?;
        let name = name_token.lexeme.to_string();
        let value = match self.lookahead.map(|t| t.kind) {
            Some(TokenKind::Comma) => Expr::Null,
            Some(TokenKind::Delimeter) => Expr::Null,
            _ => self.parse_declaration_value()?,
        };
        Ok(Expr::Set(name, Box::new(value)))
    }

    // DeclarationValue
    //  : '=' Expression
    fn parse_declaration_value(&mut self) -> Result<Expr, &'static str> {
        self.eat(TokenKind::SimpleAssign)?;
        self.parse_expression()
    }

    // IfStatement
    //  : 'if' '(' Expression ')' Statements
    //  : 'if' '(' Expression ')' Statements 'else' Statements
    fn parse_if_statement(&mut self) -> Result<Expr, &'static str> {
        self.eat(TokenKind::If)?;
        self.eat(TokenKind::LParen)?;

        let condition = self.parse_condition()?;

        self.eat(TokenKind::RParen)?;
        self.eat(TokenKind::OpeningBrace)?;

        let consequent = self.parse_statements(TokenKind::ClosingBrace)?;
        self.eat(TokenKind::ClosingBrace)?;

        let alternate = if self.lookahead.map(|t| t.kind) == Some(TokenKind::Else) {
            self.eat(TokenKind::Else)?;
            self.eat(TokenKind::OpeningBrace)?;
            let alt = self.parse_statements(TokenKind::ClosingBrace)?;
            self.eat(TokenKind::ClosingBrace)?;
            alt
        } else {
            vec![]
        };

        Ok(Expr::If(condition, consequent, alternate))
    }

    // Condition
    //  : ConditionPrimary
    //  | ConditionPrimary ('>' | '<' | '>=' | '<=' | '==' | '!=') ConditionPrimary
    fn parse_condition(&mut self) -> Result<Condition, &'static str> {
        let left = self.parse_condition_primary()?;

        // Check for comparison operator
        if let Some(token) = &self.lookahead {
            let op = match token.kind {
                TokenKind::Gt => Some(Operation::Gt),
                TokenKind::Lt => Some(Operation::Lt),
                TokenKind::Gte => Some(Operation::Gte),
                TokenKind::Lte => Some(Operation::Lte),
                TokenKind::Eq => Some(Operation::Eq),
                TokenKind::Neq => Some(Operation::Neq),
                _ => None,
            };

            if let Some(op) = op {
                self.eat(token.kind)?;
                let right = self.parse_condition_primary()?;
                return Ok(Condition::Binary(op, Box::new(left), Box::new(right)));
            }
        }

        Ok(left)
    }

    // ConditionPrimary - reuses parse_primary and converts to Condition
    fn parse_condition_primary(&mut self) -> Result<Condition, &'static str> {
        let expr = self.parse_primary()?;
        Self::expr_to_condition(expr)
    }

    // Convert an Expr to a Condition (only certain Expr variants are valid)
    fn expr_to_condition(expr: Expr) -> Result<Condition, &'static str> {
        match expr {
            Expr::Null => Ok(Condition::Null),
            Expr::Bool(b) => Ok(Condition::Bool(b)),
            Expr::Int(n) => Ok(Condition::Int(n)),
            Expr::Float(f) => Ok(Condition::Float(f)),
            Expr::Str(s) => Ok(Condition::Str(s)),
            Expr::Var(name) => Ok(Condition::Var(name)),
            _ => Err("Expression cannot be used as condition"),
        }
    }

    // BlockStatement
    //  : '{' StatementList '}'
    //  | '{' '}'
    fn parse_block_statemnt(&mut self) -> Result<Expr, &'static str> {
        self.eat(TokenKind::OpeningBrace)?;
        let expr = if self.lookahead.map(|t| t.kind) == Some(TokenKind::ClosingBrace) {
            Expr::Block(vec![])
        } else {
            Expr::Block(self.parse_statements(TokenKind::ClosingBrace)?)
        };
        self.eat(TokenKind::ClosingBrace)?;

        Ok(expr)
    }

    // ExpressionStatement
    //  : Expression ';'
    fn parse_expression_statemnt(&mut self) -> Result<Expr, &'static str> {
        let expr = self.parse_expression()?;
        self.eat(TokenKind::Delimeter)?;
        Ok(expr)
    }

    // Expression
    //  : AssignmentExpression
    fn parse_expression(&mut self) -> Result<Expr, &'static str> {
        self.parse_assignment_expression()
    }

    // AssignmentExpression
    //  : LogicalOrExpression
    //  | LeftHandSideExpression '=' AssignmentExpression
    fn parse_assignment_expression(&mut self) -> Result<Expr, &'static str> {
        let left = self.parse_logical_or_expression()?;

        if let Some(token) = &self.lookahead {
            if token.kind == TokenKind::SimpleAssign {
                self.eat(TokenKind::SimpleAssign)?;

                // Validate left-hand side is an identifier
                let name = match left {
                    Expr::Var(name) => name,
                    _ => return Err("Invalid left-hand side in assignment"),
                };

                let right = self.parse_assignment_expression()?;
                return Ok(Expr::Assign(name, Box::new(right)));
            }
        }

        Ok(left)
    }

    // LogicalOrExpression (lowest precedence of these operators)
    //  : LogicalAndExpression
    //  | LogicalOrExpression '||' LogicalAndExpression
    fn parse_logical_or_expression(&mut self) -> Result<Expr, &'static str> {
        let mut left = self.parse_logical_and_expression()?;

        while let Some(token) = &self.lookahead {
            if token.kind != TokenKind::Or {
                break;
            }
            self.eat(TokenKind::Or)?;
            let right = self.parse_logical_and_expression()?;
            left = Expr::Binary(Operation::Or, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    // LogicalAndExpression
    //  : BitwiseOrExpression
    //  | LogicalAndExpression '&&' BitwiseOrExpression
    fn parse_logical_and_expression(&mut self) -> Result<Expr, &'static str> {
        let mut left = self.parse_bitwise_or_expression()?;

        while let Some(token) = &self.lookahead {
            if token.kind != TokenKind::And {
                break;
            }
            self.eat(TokenKind::And)?;
            let right = self.parse_bitwise_or_expression()?;
            left = Expr::Binary(Operation::And, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    // BitwiseOrExpression
    //  : BitwiseAndExpression
    //  | BitwiseOrExpression '|' BitwiseAndExpression
    fn parse_bitwise_or_expression(&mut self) -> Result<Expr, &'static str> {
        let mut left = self.parse_bitwise_and_expression()?;

        while let Some(token) = &self.lookahead {
            if token.kind != TokenKind::BitwiseOr {
                break;
            }
            self.eat(TokenKind::BitwiseOr)?;
            let right = self.parse_bitwise_and_expression()?;
            left = Expr::Binary(Operation::BitwiseOr, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    // BitwiseAndExpression
    //  : AdditiveExpression
    //  | BitwiseAndExpression '&' AdditiveExpression
    fn parse_bitwise_and_expression(&mut self) -> Result<Expr, &'static str> {
        let mut left = self.parse_additive_expression()?;

        while let Some(token) = &self.lookahead {
            if token.kind != TokenKind::BitwiseAnd {
                break;
            }
            self.eat(TokenKind::BitwiseAnd)?;
            let right = self.parse_additive_expression()?;
            left = Expr::Binary(Operation::BitwiseAnd, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    // AdditiveExpression
    //  : MultiplicativeExpression
    //  | AdditiveExpression ('+' | '-') MultiplicativeExpression
    fn parse_additive_expression(&mut self) -> Result<Expr, &'static str> {
        let mut left = self.parse_multiplicative_expression()?;

        while let Some(token) = &self.lookahead {
            let op = match token.kind {
                TokenKind::Plus => Operation::Add,
                TokenKind::Minus => Operation::Sub,
                _ => break,
            };
            self.eat(token.kind)?;
            let right = self.parse_multiplicative_expression()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    // MultiplicativeExpression
    //  : PrimaryExpression
    //  | MultiplicativeExpression ('*' | '/' | '%') PrimaryExpression
    fn parse_multiplicative_expression(&mut self) -> Result<Expr, &'static str> {
        let mut left = self.parse_primary()?;

        while let Some(token) = &self.lookahead {
            // Match multiplicative operators; break on anything else
            let op = match token.kind {
                TokenKind::Star => Operation::Mul,
                TokenKind::Slash => Operation::Div,
                TokenKind::Percent => Operation::Mod,
                _ => break, // Not a multiplicative operator, exit loop
            };
            self.eat(token.kind)?;
            let right = self.parse_primary()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    // BooleanLiteral
    //  : 'true'
    //  | 'false'
    fn parse_boolean_literal(&mut self) -> Result<Expr, &'static str> {
        match self.lookahead.as_ref().map(|t| t.kind) {
            Some(TokenKind::True) => {
                self.eat(TokenKind::True)?;
                Ok(Expr::Bool(true))
            }
            Some(TokenKind::False) => {
                self.eat(TokenKind::False)?;
                Ok(Expr::Bool(false))
            }
            _ => Err("Unexpected token: expected boolean literal"),
        }
    }

    // NullLiteral
    //  : 'null'
    fn parse_null_literal(&mut self) -> Result<Expr, &'static str> {
        self.eat(TokenKind::Null)?;
        Ok(Expr::Null)
    }

    // PrimaryExpression
    //  : NumericLiteral
    //  | StringLiteral
    //  | Identifier
    //  | '(' Expression ')'
    //  | '+' PrimaryExpression  (unary plus)
    //  | '-' PrimaryExpression  (unary minus)
    fn parse_primary(&mut self) -> Result<Expr, &'static str> {
        match self.lookahead.as_ref().map(|t| t.kind) {
            Some(TokenKind::Number) => self.parse_numeric_literal(),
            Some(TokenKind::String) => self.parse_string_literal(),
            Some(TokenKind::True) | Some(TokenKind::False) => self.parse_boolean_literal(),
            Some(TokenKind::Null) => self.parse_null_literal(),
            Some(TokenKind::Identifier) => self.parse_identifier(),
            Some(TokenKind::LParen) => {
                self.eat(TokenKind::LParen)?;
                let expr = self.parse_expression()?;
                self.eat(TokenKind::RParen)?;
                Ok(expr)
            }
            Some(TokenKind::Plus) => {
                // Unary plus - just return the operand (no-op)
                self.eat(TokenKind::Plus)?;
                self.parse_primary()
            }
            Some(TokenKind::Minus) => {
                // Unary minus - represent as (0 - operand)
                self.eat(TokenKind::Minus)?;
                let operand = self.parse_primary()?;
                Ok(Expr::Binary(
                    Operation::Sub,
                    Box::new(Expr::Int(0)),
                    Box::new(operand),
                ))
            }
            _ => Err("Unexpected token: expected literal or '('"),
        }
    }

    // Identifier
    //  : IDENTIFIER
    fn parse_identifier(&mut self) -> Result<Expr, &'static str> {
        let token = self.eat(TokenKind::Identifier)?;
        Ok(Expr::Var(token.lexeme.to_string()))
    }

    // NumericLiteral
    //  : NUMBER
    fn parse_numeric_literal(&mut self) -> Result<Expr, &'static str> {
        let token = self.eat(TokenKind::Number)?;
        let lexeme = token.lexeme;

        if let Ok(i) = lexeme.parse::<i64>() {
            Ok(Expr::Int(i))
        } else if let Ok(f) = lexeme.parse::<f64>() {
            Ok(Expr::Float(f))
        } else {
            Err("Invalid number literal")
        }
    }

    // StringLiteral
    //  : STRING
    fn parse_string_literal(&mut self) -> Result<Expr, &'static str> {
        let token = self.eat(TokenKind::String)?;
        Ok(Expr::Str(token.lexeme.to_string()))
    }
}
