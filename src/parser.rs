use crate::{Condition, Expr, Operation};

#[derive(Debug, PartialEq, Clone)]
enum Token {
    LParen,
    RParen,
    Symbol(String),
    Number(String),
    String(String),
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(input: &str) -> Result<Self, String> {
        let tokens = Self::tokenize(input)?;
        Ok(Self { tokens, pos: 0 })
    }

    fn tokenize(input: &str) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&ch) = chars.peek() {
            match ch {
                ' ' | '\t' | '\n' | '\r' => {
                    chars.next();
                }
                '(' => {
                    tokens.push(Token::LParen);
                    chars.next();
                }
                ')' => {
                    tokens.push(Token::RParen);
                    chars.next();
                }
                '"' => {
                    chars.next(); // consume opening quote
                    let mut string_val = String::new();
                    loop {
                        match chars.next() {
                            Some('"') => break,
                            Some(ch) => string_val.push(ch),
                            None => return Err("Unterminated string".to_string()),
                        }
                    }
                    tokens.push(Token::String(string_val));
                }
                _ if ch.is_ascii_digit() => {
                    let mut num = String::new();
                    num.push(chars.next().unwrap());
                    
                    while let Some(&ch) = chars.peek() {
                        if ch.is_ascii_digit() || ch == '.' {
                            num.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Number(num));
                }
                '-' => {
                    // Check if it's a negative number or minus operator
                    chars.next(); // consume '-'
                    if let Some(&next_ch) = chars.peek() {
                        if next_ch.is_ascii_digit() {
                            // It's a negative number
                            let mut num = String::from("-");
                            while let Some(&ch) = chars.peek() {
                                if ch.is_ascii_digit() || ch == '.' {
                                    num.push(chars.next().unwrap());
                                } else {
                                    break;
                                }
                            }
                            tokens.push(Token::Number(num));
                        } else {
                            // It's a minus operator
                            tokens.push(Token::Symbol(String::from("-")));
                        }
                    } else {
                        // End of input, treat as symbol
                        tokens.push(Token::Symbol(String::from("-")));
                    }
                }
                _ => {
                    let mut symbol = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_whitespace() || ch == '(' || ch == ')' {
                            break;
                        }
                        symbol.push(chars.next().unwrap());
                    }
                    tokens.push(Token::Symbol(symbol));
                }
            }
        }

        Ok(tokens)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn consume(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(token)
        } else {
            None
        }
    }

    pub fn parse(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(Token::LParen) => self.parse_list(),
            Some(Token::Number(n)) => {
                let num = n.clone();
                self.consume();
                if num.contains('.') {
                    Ok(Expr::Float(num.parse().map_err(|_| "Invalid float")?))
                } else {
                    Ok(Expr::Int(num.parse().map_err(|_| "Invalid integer")?))
                }
            }
            Some(Token::String(s)) => {
                let string = s.clone();
                self.consume();
                Ok(Expr::Str(string))
            }
            Some(Token::Symbol(s)) => {
                let sym = s.clone();
                self.consume();
                match sym.as_str() {
                    "null" => Ok(Expr::Null),
                    "true" => Ok(Expr::Bool(true)),
                    "false" => Ok(Expr::Bool(false)),
                    _ => Ok(Expr::Var(sym)),
                }
            }
            _ => Err("Unexpected token".to_string()),
        }
    }

    fn parse_list(&mut self) -> Result<Expr, String> {
        self.consume(); // consume '('

        let op = match self.consume() {
            Some(Token::Symbol(s)) => s,
            _ => return Err("Expected operator".to_string()),
        };

        let expr = match op.as_str() {
            "+" => self.parse_binary(Operation::Add)?,
            "-" => self.parse_binary(Operation::Sub)?,
            "*" => self.parse_binary(Operation::Mul)?,
            "/" => self.parse_binary(Operation::Div)?,
            ">" => self.parse_binary(Operation::Gt)?,
            "<" => self.parse_binary(Operation::Lt)?,
            ">=" => self.parse_binary(Operation::Gte)?,
            "<=" => self.parse_binary(Operation::Lte)?,
            "==" => self.parse_binary(Operation::Eq)?,
            "!=" => self.parse_binary(Operation::Neq)?,
            "set" => self.parse_set()?,
            "assign" => self.parse_assign()?,
            "block" => self.parse_block()?,
            "if" => self.parse_if()?,
            "while" => self.parse_while()?,
            "fn" => self.parse_function()?,
            _ => {
                // If not a keyword, treat as function call
                self.pos -= 1; // put back the symbol
                self.parse_function_call()?
            }
        };

        match self.consume() {
            Some(Token::RParen) => Ok(expr),
            _ => Err("Expected ')'".to_string()),
        }
    }

    fn parse_binary(&mut self, op: Operation) -> Result<Expr, String> {
        let left = Box::new(self.parse()?);
        let right = Box::new(self.parse()?);
        Ok(Expr::Binary(op, left, right))
    }

    fn parse_set(&mut self) -> Result<Expr, String> {
        let name = match self.consume() {
            Some(Token::Symbol(s)) => s,
            _ => return Err("Expected variable name".to_string()),
        };
        let value = Box::new(self.parse()?);
        Ok(Expr::Set(name, value))
    }

    fn parse_assign(&mut self) -> Result<Expr, String> {
        let name = match self.consume() {
            Some(Token::Symbol(s)) => s,
            _ => return Err("Expected variable name".to_string()),
        };
        let value = Box::new(self.parse()?);
        Ok(Expr::Assign(name, value))
    }

    fn parse_block(&mut self) -> Result<Expr, String> {
        let mut exprs = Vec::new();
        while let Some(token) = self.peek() {
            if *token == Token::RParen {
                break;
            }
            exprs.push(self.parse()?);
        }
        Ok(Expr::Block(exprs))
    }

    fn parse_if(&mut self) -> Result<Expr, String> {
        let condition = self.parse_condition()?;
        
        // Parse then branch
        let mut then_branch = Vec::new();
        match self.consume() {
            Some(Token::LParen) => {
                // Block syntax: (if condition (...) (...))
                self.pos -= 1; // put back the LParen
                let then_expr = self.parse()?;
                if let Expr::Block(exprs) = then_expr {
                    then_branch = exprs;
                } else {
                    then_branch.push(then_expr);
                }
            }
            _ => {
                self.pos -= 1;
                then_branch.push(self.parse()?);
            }
        }
        
        // Parse else branch
        let mut else_branch = Vec::new();
        match self.consume() {
            Some(Token::LParen) => {
                self.pos -= 1;
                let else_expr = self.parse()?;
                if let Expr::Block(exprs) = else_expr {
                    else_branch = exprs;
                } else {
                    else_branch.push(else_expr);
                }
            }
            _ => {
                self.pos -= 1;
                else_branch.push(self.parse()?);
            }
        }
        
        // Validate branches are not empty
        if then_branch.is_empty() {
            return Err("If statement requires non-empty then branch".to_string());
        }
        if else_branch.is_empty() {
            return Err("If statement requires non-empty else branch".to_string());
        }
        
        Ok(Expr::If(condition, then_branch, else_branch))
    }

    fn parse_while(&mut self) -> Result<Expr, String> {
        let condition = self.parse_condition()?;
        
        // Parse body
        let mut body = Vec::new();
        while let Some(token) = self.peek() {
            if *token == Token::RParen {
                break;
            }
            body.push(self.parse()?);
        }
        
        // Validate body is not empty
        if body.is_empty() {
            return Err("While loop requires non-empty body".to_string());
        }
        
        Ok(Expr::While(condition, body))
    }

    fn parse_condition(&mut self) -> Result<Condition, String> {
        match self.peek() {
            Some(Token::LParen) => self.parse_condition_list(),
            Some(Token::Number(n)) => {
                let num = n.clone();
                self.consume();
                if num.contains('.') {
                    Ok(Condition::Float(num.parse().map_err(|_| "Invalid float")?))
                } else {
                    Ok(Condition::Int(num.parse().map_err(|_| "Invalid integer")?))
                }
            }
            Some(Token::String(s)) => {
                let string = s.clone();
                self.consume();
                Ok(Condition::Str(string))
            }
            Some(Token::Symbol(s)) => {
                let sym = s.clone();
                self.consume();
                match sym.as_str() {
                    "null" => Ok(Condition::Null),
                    "true" => Ok(Condition::Bool(true)),
                    "false" => Ok(Condition::Bool(false)),
                    _ => Ok(Condition::Var(sym)),
                }
            }
            _ => Err("Unexpected token in condition".to_string()),
        }
    }

    fn parse_condition_list(&mut self) -> Result<Condition, String> {
        self.consume(); // consume '('

        let op = match self.consume() {
            Some(Token::Symbol(s)) => s,
            _ => return Err("Expected operator in condition".to_string()),
        };

        let condition = match op.as_str() {
            "+" => self.parse_condition_binary(Operation::Add)?,
            "-" => self.parse_condition_binary(Operation::Sub)?,
            "*" => self.parse_condition_binary(Operation::Mul)?,
            "/" => self.parse_condition_binary(Operation::Div)?,
            ">" => self.parse_condition_binary(Operation::Gt)?,
            "<" => self.parse_condition_binary(Operation::Lt)?,
            ">=" => self.parse_condition_binary(Operation::Gte)?,
            "<=" => self.parse_condition_binary(Operation::Lte)?,
            "==" => self.parse_condition_binary(Operation::Eq)?,
            "!=" => self.parse_condition_binary(Operation::Neq)?,
            _ => {
                // If not a keyword, treat as function call
                self.pos -= 1; // put back the symbol
                self.parse_condition_function_call()?
            }
        };

        match self.consume() {
            Some(Token::RParen) => Ok(condition),
            _ => Err("Expected ')' in condition".to_string()),
        }
    }

    fn parse_condition_binary(&mut self, op: Operation) -> Result<Condition, String> {
        let left = Box::new(self.parse_condition()?);
        let right = Box::new(self.parse_condition()?);
        Ok(Condition::Binary(op, left, right))
    }

    fn parse_function(&mut self) -> Result<Expr, String> {
        // Expect (fn (params...) body...)
        // Parse parameter list
        match self.consume() {
            Some(Token::LParen) => {}
            _ => return Err("Expected '(' after 'fn'".to_string()),
        }
        
        let mut params = Vec::new();
        while let Some(token) = self.peek() {
            if *token == Token::RParen {
                self.consume(); // consume ')'
                break;
            }
            match self.consume() {
                Some(Token::Symbol(s)) => params.push(s),
                _ => return Err("Expected parameter name".to_string()),
            }
        }
        
        // Parse body (rest of the expressions until closing paren)
        let mut body = Vec::new();
        while let Some(token) = self.peek() {
            if *token == Token::RParen {
                break;
            }
            body.push(self.parse()?);
        }
        
        if body.is_empty() {
            return Err("Function requires non-empty body".to_string());
        }
        
        Ok(Expr::Function(params, body))
    }

    fn parse_function_call(&mut self) -> Result<Expr, String> {
        // Expect (funcname args...)
        let name = match self.consume() {
            Some(Token::Symbol(s)) => s,
            _ => return Err("Expected function name".to_string()),
        };
        
        let mut args = Vec::new();
        while let Some(token) = self.peek() {
            if *token == Token::RParen {
                break;
            }
            args.push(self.parse()?);
        }
        
        Ok(Expr::FunctionCall(name, args))
    }

    fn parse_condition_function_call(&mut self) -> Result<Condition, String> {
        // Expect (funcname args...)
        let name = match self.consume() {
            Some(Token::Symbol(s)) => s,
            _ => return Err("Expected function name in condition".to_string()),
        };
        
        let mut args = Vec::new();
        while let Some(token) = self.peek() {
            if *token == Token::RParen {
                break;
            }
            args.push(self.parse_condition()?);
        }
        
        Ok(Condition::FunctionCall(name, args))
    }
}
