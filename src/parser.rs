use crate::{Condition, Expr, Operation};
use crate::tokeniser::{Token, TokenKind, Tokeniser};

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

        Ok(Expr::Null)
    }

    fn eat(&mut self, expected_token: TokenKind ) -> Result<Token, &'static str> {
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
       Ok(Expr::Block(self.literal()?))
    }

    // Liternal
    //    : NumericLiteral | StringLiteral
    fn parse_literal(&mut self) -> Result<Expr, &'static str> {
        
    }

    // NumbericLiternal
    //    : NUMBER
    fn parse_numeric_literal(&mut self) {

    }

}
