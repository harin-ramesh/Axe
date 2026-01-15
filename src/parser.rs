use crate::tokeniser::{Token, TokenKind, Tokeniser};
use crate::{Expr, Operation};

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
    fn parse_statement(&mut self) -> Result<Expr, &'static str> {
        let expr = if self.lookahead.map(|t| t.kind) == Some(TokenKind::OpeningBrace) {
            self.parse_block_statemnt()?
        } else {
            self.parse_expression_statemnt()?
        };
        Ok(expr)
    }

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
    //  : AdditiveExpression
    fn parse_expression(&mut self) -> Result<Expr, &'static str> {
        self.parse_additive_expression()
    }

    // AdditiveExpression
    //  : PrimaryExpression
    //  | AdditiveExpression '+' PrimaryExpression
    fn parse_additive_expression(&mut self) -> Result<Expr, &'static str> {
        let mut left = self.parse_primary()?;

        while let Some(token) = &self.lookahead {
            if token.kind == TokenKind::Plus {
                self.eat(TokenKind::Plus)?;
                let right = self.parse_primary()?;
                left = Expr::Binary(Operation::Add, Box::new(left), Box::new(right));
            } else {
                break;
            }
        }

        Ok(left)
    }

    // PrimaryExpression
    //  : NumericLiteral
    //  | StringLiteral
    //  | '(' Expression ')'
    //  | '+' PrimaryExpression  (unary plus)
    fn parse_primary(&mut self) -> Result<Expr, &'static str> {
        match self.lookahead.as_ref().map(|t| t.kind) {
            Some(TokenKind::Number) => self.parse_numeric_literal(),
            Some(TokenKind::String) => self.parse_string_literal(),
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
            _ => Err("Unexpected token: expected literal or '('"),
        }
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
