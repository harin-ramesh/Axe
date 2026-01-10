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

    fn parse_expression_statemnt(&mut self) -> Result<Expr, &'static str> {
        let expr = self.parse_expression()?;
        self.eat(TokenKind::Delimeter)?;
        Ok(expr)
    }

    fn parse_expression(&mut self) -> Result<Expr, &'static str> {
        Ok(self.parse_literal()?)
    }

    fn parse_literal(&mut self) -> Result<Expr, &'static str> {
        match self.lookahead.as_ref().map(|t| t.kind) {
            Some(TokenKind::Number) => self.parse_numeric_literal(),
            Some(TokenKind::String) => self.parse_string_literal(),
            _ => Err("Unexpected token: expected literal"),
        }
    }

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

    fn parse_string_literal(&mut self) -> Result<Expr, &'static str> {
        let token = self.eat(TokenKind::String)?;
        Ok(Expr::Str(token.lexeme.to_string()))
    }
}
