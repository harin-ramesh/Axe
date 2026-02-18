use crate::ast::{Expr, ExprKind, Literal, Operation, Program, Stmt, UnaryOp};
use crate::context::Context;
use crate::interner::Symbol;
use crate::tokeniser::{Token, TokenKind, Tokeniser};

/// A variable declaration: (name, initializer, target_object)
type Declaration = (Symbol, Option<Expr>, Option<Expr>);

pub struct Parser<'src, 'ctx> {
    tokeniser: Tokeniser<'src>,
    lookahead: Option<Token<'src>>,
    ctx: &'ctx Context,
}

impl<'src, 'ctx> Parser<'src, 'ctx> {
    pub fn new(input: &'src str, ctx: &'ctx Context) -> Self {
        let tokeniser = Tokeniser::new(input);
        Self {
            tokeniser,
            lookahead: None,
            ctx,
        }
    }

    pub fn parse(&mut self) -> Result<Program, &'static str> {
        self.lookahead = Some(self.tokeniser.get_next_token()?);
        let expr = self.parse_program()?;
        Ok(expr)
    }

    /// Intern a string using the context's interner.
    #[inline]
    fn intern(&self, s: &str) -> Symbol {
        self.ctx.intern(s)
    }

    fn eat(&mut self, expected_token: TokenKind) -> Result<Token<'src>, &'static str> {
        let token = match self.lookahead.take() {
            None => return Err("Unexpected end of input"),
            Some(t) => t,
        };

        if token.kind != expected_token {
            let msg = format!(
                "[Line {}] Unexpected `{}`, expected `{}`",
                token.line, token.lexeme, expected_token
            );
            let leaked: &'static str = Box::leak(msg.into_boxed_str());
            return Err(leaked);
        }

        self.lookahead = Some(self.tokeniser.get_next_token()?);
        Ok(token)
    }

    fn parse_program(&mut self) -> Result<Program, &'static str> {
        let stmts = self.parse_statements(TokenKind::Eof)?;
        Ok(Program { stmts })
    }

    // StatementList
    //  : Statement
    //  | StatemtnList Statement -> Statement Statement Statment
    fn parse_statements(&mut self, stop_token: TokenKind) -> Result<Vec<Stmt>, &'static str> {
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
    //  | ForStatement
    //  | WhileStatement
    //  | FunctionDeclaration
    //  | ClassDeclaration
    //  | ReturnStatement
    //  | Break
    //  | Continue
    //  | From
    fn parse_statement(&mut self) -> Result<Stmt, &'static str> {
        let expr = match self.lookahead.map(|t| t.kind) {
            Some(TokenKind::OpeningBrace) => self.parse_block_statemnt()?,
            Some(TokenKind::Let) => self.parse_let_statement()?,
            Some(TokenKind::If) => self.parse_if_statement()?,
            Some(TokenKind::While) => self.parse_while_statement()?,
            Some(TokenKind::For) => self.parse_for_statement()?,
            Some(TokenKind::Fn) => self.parse_function_declaration()?,
            Some(TokenKind::Class) => self.parse_class_declaration()?,
            Some(TokenKind::Return) => self.parse_return_statement()?,
            Some(TokenKind::Break) => {
                self.eat(TokenKind::Break)?;
                self.eat(TokenKind::Delimeter)?;
                Stmt::Break
            }
            Some(TokenKind::Continue) => {
                self.eat(TokenKind::Continue)?;
                self.eat(TokenKind::Delimeter)?;
                Stmt::Continue
            }
            Some(TokenKind::From) => self.parse_from_statement()?,
            _ => self.parse_expression_statemnt()?,
        };
        Ok(expr)
    }

    fn parse_from_statement(&mut self) -> Result<Stmt, &'static str> {
        self.eat(TokenKind::From)?;
        let module_token = self.eat(TokenKind::Identifier)?;
        let module_name = self.intern(module_token.lexeme);
        self.eat(TokenKind::Import)?;
        let mut imports = Vec::new();
        loop {
            let import_token = self.eat(TokenKind::Identifier)?;
            imports.push(self.intern(import_token.lexeme));

            if self.lookahead.map(|t| t.kind) == Some(TokenKind::Comma) {
                self.eat(TokenKind::Comma)?;
            } else {
                break;
            }
        }
        self.eat(TokenKind::Delimeter)?;
        Ok(Stmt::Import(module_name, imports))
    }

    // ReturnStatement
    //  : 'return' ';'
    //  | 'return' Expression ';'
    fn parse_return_statement(&mut self) -> Result<Stmt, &'static str> {
        self.eat(TokenKind::Return)?;
        // Handle bare `return;` with no expression
        let return_expr = if self.lookahead.map(|t| t.kind) == Some(TokenKind::Delimeter) {
            Expr::Literal(Literal::Null)
        } else {
            self.parse_logical_or_expression()?
        };
        self.eat(TokenKind::Delimeter)?;
        Ok(Stmt::Return(Box::new(return_expr)))
    }

    // ClassDeclaration
    //  : 'class' Identifier (':' Identifier)? '{' Statements '}'
    fn parse_class_declaration(&mut self) -> Result<Stmt, &'static str> {
        self.eat(TokenKind::Class)?;
        let name_token = self.eat(TokenKind::Identifier)?;
        let name = self.intern(name_token.lexeme);
        let parent = if self.lookahead.map(|t| t.kind) == Some(TokenKind::Colon) {
            self.eat(TokenKind::Colon)?;
            let parent_token = self.eat(TokenKind::Identifier)?;
            Some(self.intern(parent_token.lexeme))
        } else {
            None
        };
        self.eat(TokenKind::OpeningBrace)?;
        let body = self.parse_statements(TokenKind::ClosingBrace)?;
        self.eat(TokenKind::ClosingBrace)?;
        Ok(Stmt::Class(name, parent, body))
    }

    // FunctionDeclaration
    //  : 'fn' Identifier '(' ParameterList ')' '{' Statements '}'
    fn parse_function_declaration(&mut self) -> Result<Stmt, &'static str> {
        self.eat(TokenKind::Fn)?;

        let name_token = self.eat(TokenKind::Identifier)?;
        let name = self.intern(name_token.lexeme);

        self.eat(TokenKind::LParen)?;
        let params = self.parse_parameter_list()?;
        self.eat(TokenKind::RParen)?;

        self.eat(TokenKind::OpeningBrace)?;
        let body = self.parse_statements(TokenKind::ClosingBrace)?;
        self.eat(TokenKind::ClosingBrace)?;

        Ok(Stmt::Function(name, params, Box::new(Stmt::Block(body))))
    }

    // ParameterList
    //  : Identifier (',' Identifier)*
    //  | ε
    fn parse_parameter_list(&mut self) -> Result<Vec<Symbol>, &'static str> {
        let mut params = Vec::new();

        // Parse first parameter if present
        if self.lookahead.map(|t| t.kind) == Some(TokenKind::Identifier) {
            let token = self.eat(TokenKind::Identifier)?;
            params.push(self.intern(token.lexeme));

            // Parse remaining parameters
            while self.lookahead.map(|t| t.kind) == Some(TokenKind::Comma) {
                self.eat(TokenKind::Comma)?;
                let token = self.eat(TokenKind::Identifier)?;
                params.push(self.intern(token.lexeme));
            }
        }

        Ok(params)
    }

    // WhileStatement
    //  : 'while' '(' Expression ')' Statements
    fn parse_while_statement(&mut self) -> Result<Stmt, &'static str> {
        self.eat(TokenKind::While)?;

        self.eat(TokenKind::LParen)?;
        let condition = self.parse_condition()?;
        self.eat(TokenKind::RParen)?;

        self.eat(TokenKind::OpeningBrace)?;
        let body = self.parse_statements(TokenKind::ClosingBrace)?;
        self.eat(TokenKind::ClosingBrace)?;

        Ok(Stmt::While(condition, Box::new(Stmt::Block(body))))
    }

    // ForStatement
    //  : 'for' Identifier 'in' Expression '{' Statements '}'
    fn parse_for_statement(&mut self) -> Result<Stmt, &'static str> {
        self.eat(TokenKind::For)?;

        // Parse loop variable name
        let var_token = self.eat(TokenKind::Identifier)?;
        let var_name = self.intern(var_token.lexeme);

        self.eat(TokenKind::In)?;

        // Parse iterable expression (e.g., range(1, 10) or a list variable)
        let iterable = self.parse_logical_or_expression()?;

        self.eat(TokenKind::OpeningBrace)?;
        let body = self.parse_statements(TokenKind::ClosingBrace)?;
        self.eat(TokenKind::ClosingBrace)?;

        Ok(Stmt::For(var_name, iterable, Box::new(Stmt::Block(body))))
    }

    // LetStatement
    //  : 'let' DeclarationList ';'
    fn parse_let_statement(&mut self) -> Result<Stmt, &'static str> {
        self.eat(TokenKind::Let)?;
        let declarations = self.parse_declarations()?;
        self.eat(TokenKind::Delimeter)?;
        Ok(Stmt::Let(declarations))
    }

    // DeclarationList
    //  : Declaration
    //  | DeclarationList ',' Declaration
    fn parse_declarations(&mut self) -> Result<Vec<Declaration>, &'static str> {
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
    fn parse_declaration(&mut self) -> Result<Declaration, &'static str> {
        let name_token = self.eat(TokenKind::Identifier)?;
        let name = self.intern(name_token.lexeme);
        let value = match self.lookahead.map(|t| t.kind) {
            Some(TokenKind::Comma) => Expr::Literal(Literal::Null),
            Some(TokenKind::Delimeter) => Expr::Literal(Literal::Null),
            _ => self.parse_declaration_value()?,
        };
        Ok((name, Some(value), None))
    }

    // DeclarationValue
    //  : '=' Expression
    fn parse_declaration_value(&mut self) -> Result<Expr, &'static str> {
        self.eat(TokenKind::SimpleAssign)?;
        self.parse_logical_or_expression()
    }

    // IfStatement
    //  : 'if' '(' Expression ')' Statements
    //  : 'if' '(' Expression ')' Statements 'else' Statements
    fn parse_if_statement(&mut self) -> Result<Stmt, &'static str> {
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

        Ok(Stmt::If(
            condition,
            Box::new(Stmt::Block(consequent)),
            Box::new(Stmt::Block(alternate)),
        ))
    }

    // Condition
    //  : EqualityExpression (supports all comparison and equality operators)
    fn parse_condition(&mut self) -> Result<Expr, &'static str> {
        self.parse_equality_expression()
    }

    // BlockStatement
    //  : '{' StatementList '}'
    //  | '{' '}'
    fn parse_block_statemnt(&mut self) -> Result<Stmt, &'static str> {
        self.eat(TokenKind::OpeningBrace)?;
        let block = if self.lookahead.map(|t| t.kind) == Some(TokenKind::ClosingBrace) {
            Stmt::Block(vec![])
        } else {
            Stmt::Block(self.parse_statements(TokenKind::ClosingBrace)?)
        };
        self.eat(TokenKind::ClosingBrace)?;

        Ok(block)
    }

    // ExpressionStatement
    //  : Expression ';'
    fn parse_expression_statemnt(&mut self) -> Result<Stmt, &'static str> {
        let expr = self.parse_expression()?;
        self.eat(TokenKind::Delimeter)?;
        Ok(expr)
    }

    // Expression
    //  : AssignmentExpression
    fn parse_expression(&mut self) -> Result<Stmt, &'static str> {
        self.parse_assignment_expression()
    }

    // AssignmentExpression
    //  : LogicalOrExpression
    //  | LeftHandSideExpression '=' AssignmentExpression
    fn parse_assignment_expression(&mut self) -> Result<Stmt, &'static str> {
        let left = self.parse_logical_or_expression()?;

        if let Some(token) = &self.lookahead
            && token.kind == TokenKind::SimpleAssign
        {
            self.eat(TokenKind::SimpleAssign)?;

            // Validate left-hand side is an identifier
            match &left.kind {
                ExprKind::Var(name) => {
                    let right = self.parse_logical_or_expression()?;
                    return Ok(Stmt::Assign(name.clone(), right));
                }
                ExprKind::Property(obj_expr, prop_name) => {
                    let right = self.parse_logical_or_expression()?;
                    return Ok(Stmt::Let(vec![(
                        prop_name.clone(),
                        Some(right),
                        Some(obj_expr.as_ref().clone()),
                    )]));
                }
                _ => return Err("Invalid left-hand side in assignment"),
            };
        }

        Ok(Stmt::Expr(left))
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
    //  : EqualityExpression
    //  | BitwiseAndExpression '&' EqualityExpression
    fn parse_bitwise_and_expression(&mut self) -> Result<Expr, &'static str> {
        let mut left = self.parse_equality_expression()?;

        while let Some(token) = &self.lookahead {
            if token.kind != TokenKind::BitwiseAnd {
                break;
            }
            self.eat(TokenKind::BitwiseAnd)?;
            let right = self.parse_equality_expression()?;
            left = Expr::Binary(Operation::BitwiseAnd, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    // EqualityExpression
    //  : RelationalExpression
    //  | EqualityExpression ('==' | '!=') RelationalExpression
    fn parse_equality_expression(&mut self) -> Result<Expr, &'static str> {
        let mut left = self.parse_relational_expression()?;

        while let Some(token) = &self.lookahead {
            let op = match token.kind {
                TokenKind::Eq => Operation::Eq,
                TokenKind::Neq => Operation::Neq,
                _ => break,
            };
            self.eat(token.kind)?;
            let right = self.parse_relational_expression()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    // RelationalExpression
    //  : AdditiveExpression
    //  | RelationalExpression ('<' | '>' | '<=' | '>=') AdditiveExpression
    fn parse_relational_expression(&mut self) -> Result<Expr, &'static str> {
        let mut left = self.parse_additive_expression()?;

        while let Some(token) = &self.lookahead {
            let op = match token.kind {
                TokenKind::Lt => Operation::Lt,
                TokenKind::Gt => Operation::Gt,
                TokenKind::Lte => Operation::Lte,
                TokenKind::Gte => Operation::Gte,
                _ => break,
            };
            self.eat(token.kind)?;
            let right = self.parse_additive_expression()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
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
    //  : UnaryExpression
    //  | MultiplicativeExpression ('*' | '/' | '%') UnaryExpression
    fn parse_multiplicative_expression(&mut self) -> Result<Expr, &'static str> {
        let mut left = self.parse_unary_expression()?;

        while let Some(token) = &self.lookahead {
            // Match multiplicative operators; break on anything else
            let op = match token.kind {
                TokenKind::Star => Operation::Mul,
                TokenKind::Slash => Operation::Div,
                TokenKind::Percent => Operation::Mod,
                _ => break, // Not a multiplicative operator, exit loop
            };
            self.eat(token.kind)?;
            let right = self.parse_unary_expression()?;
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
                Ok(Expr::Literal(Literal::Bool(true)))
            }
            Some(TokenKind::False) => {
                self.eat(TokenKind::False)?;
                Ok(Expr::Literal(Literal::Bool(false)))
            }
            _ => Err("Unexpected token: expected boolean literal"),
        }
    }

    // NullLiteral
    //  : 'null'
    fn parse_null_literal(&mut self) -> Result<Expr, &'static str> {
        self.eat(TokenKind::Null)?;
        Ok(Expr::Literal(Literal::Null))
    }

    // UnaryExpression
    //  : PrimaryExpression
    //  | '+' UnaryExpression  (unary plus - no-op)
    //  | '-' UnaryExpression  (unary minus / negation)
    //  | '!' UnaryExpression  (logical not)
    //  | '~' UnaryExpression  (bitwise invert)
    fn parse_unary_expression(&mut self) -> Result<Expr, &'static str> {
        match self.lookahead.as_ref().map(|t| t.kind) {
            Some(TokenKind::Plus) => {
                // Unary plus - just return the operand (no-op)
                self.eat(TokenKind::Plus)?;
                self.parse_unary_expression()
            }
            Some(TokenKind::Minus) => {
                // Unary minus (negation)
                self.eat(TokenKind::Minus)?;
                let operand = self.parse_unary_expression()?;
                Ok(Expr::Unary(UnaryOp::Neg, Box::new(operand)))
            }
            Some(TokenKind::Bang) => {
                // Logical not
                self.eat(TokenKind::Bang)?;
                let operand = self.parse_unary_expression()?;
                Ok(Expr::Unary(UnaryOp::Not, Box::new(operand)))
            }
            Some(TokenKind::Tilde) => {
                // Bitwise invert
                self.eat(TokenKind::Tilde)?;
                let operand = self.parse_unary_expression()?;
                Ok(Expr::Unary(UnaryOp::Inv, Box::new(operand)))
            }
            _ => self.parse_primary(),
        }
    }

    // PrimaryExpression
    //  : NumericLiteral
    //  | StringLiteral
    //  | Identifier
    //  | '(' Expression ')'
    fn parse_primary(&mut self) -> Result<Expr, &'static str> {
        let expr = match self.lookahead.as_ref().map(|t| t.kind) {
            Some(TokenKind::Number) => self.parse_numeric_literal()?,
            Some(TokenKind::String) => self.parse_string_literal()?,
            Some(TokenKind::True) | Some(TokenKind::False) => self.parse_boolean_literal()?,
            Some(TokenKind::Null) => self.parse_null_literal()?,
            Some(TokenKind::New) => self.parse_object_instantiation()?,
            Some(TokenKind::Identifier) => {
                let expr = self.parse_identifier()?;
                self.parse_static_access(expr)?
            }
            Some(TokenKind::LBracket) => self.parse_list_literal()?,
            Some(TokenKind::LParen) => {
                self.eat(TokenKind::LParen)?;
                let expr = self.parse_logical_or_expression()?;
                self.eat(TokenKind::RParen)?;
                expr
            }
            _ => return Err("Unexpected token: expected literal or '('"),
        };

        // Handle method/property access on all primaries: "hello".len(), foo.bar, etc.
        self.parse_member_access(expr)
    }

    fn parse_object_instantiation(&mut self) -> Result<Expr, &'static str> {
        self.eat(TokenKind::New)?;

        let class_token = self.eat(TokenKind::Identifier)?;
        let class_name = self.intern(class_token.lexeme);

        self.eat(TokenKind::LParen)?;
        let args = self.parse_argument_list()?;
        self.eat(TokenKind::RParen)?;

        Ok(Expr::New(class_name, args))
    }

    // ListLiteral
    //  : '[' ']'
    //  | '[' Expression (',' Expression)* ']'
    fn parse_list_literal(&mut self) -> Result<Expr, &'static str> {
        self.eat(TokenKind::LBracket)?;

        let mut elements = Vec::new();

        // Check for empty list
        if self.lookahead.map(|t| t.kind) != Some(TokenKind::RBracket) {
            // Parse first element
            elements.push(self.parse_logical_or_expression()?);

            // Parse remaining elements
            while self.lookahead.map(|t| t.kind) == Some(TokenKind::Comma) {
                self.eat(TokenKind::Comma)?;
                elements.push(self.parse_logical_or_expression()?);
            }
        }

        self.eat(TokenKind::RBracket)?;
        Ok(Expr::List(elements))
    }

    fn parse_static_access(&mut self, mut expr: Expr) -> Result<Expr, &'static str> {
        if self.lookahead.map(|t| t.kind) == Some(TokenKind::StaticAccess) {
            self.eat(TokenKind::StaticAccess)?;
            let property_token = self.eat(TokenKind::Identifier)?;
            let property_name = self.intern(property_token.lexeme);
            if self.lookahead.map(|t| t.kind) == Some(TokenKind::LParen) {
                self.eat(TokenKind::LParen)?;
                let args = self.parse_argument_list()?;
                self.eat(TokenKind::RParen)?;
                expr = Expr::StaticMethodCall(Box::new(expr), property_name, args);
            } else {
                expr = Expr::StaticProperty(Box::new(expr), property_name);
            }
        }
        Ok(expr)
    }

    // Parse chained property/method access: .foo.bar.baz or .foo().bar()
    fn parse_member_access(&mut self, mut expr: Expr) -> Result<Expr, &'static str> {
        while self.lookahead.map(|t| t.kind) == Some(TokenKind::MemberAccess) {
            self.eat(TokenKind::MemberAccess)?;
            let property_token = self.eat(TokenKind::Identifier)?;
            let property_name = self.intern(property_token.lexeme);

            // Check if this is a method call: .method(args)
            if self.lookahead.map(|t| t.kind) == Some(TokenKind::LParen) {
                self.eat(TokenKind::LParen)?;
                let args = self.parse_argument_list()?;
                self.eat(TokenKind::RParen)?;
                expr = Expr::MethodCall(Box::new(expr), property_name, args);
            } else {
                expr = Expr::Property(Box::new(expr), property_name);
            }
        }
        Ok(expr)
    }

    // Identifier or FunctionCall
    //  : IDENTIFIER
    //  | IDENTIFIER '(' ArgumentList? ')'
    fn parse_identifier(&mut self) -> Result<Expr, &'static str> {
        let token = self.eat(TokenKind::Identifier)?;
        let name = self.intern(token.lexeme);

        // Check if this is a function call
        if self.lookahead.map(|t| t.kind) == Some(TokenKind::LParen) {
            self.eat(TokenKind::LParen)?;
            let args = self.parse_argument_list()?;
            self.eat(TokenKind::RParen)?;
            Ok(Expr::Call(name, args))
        } else {
            Ok(Expr::Var(name))
        }
    }

    // ArgumentList
    //  : Expression (',' Expression)*
    //  | ε
    fn parse_argument_list(&mut self) -> Result<Vec<Expr>, &'static str> {
        let mut args = Vec::new();

        // Check for empty argument list
        if self.lookahead.map(|t| t.kind) == Some(TokenKind::RParen) {
            return Ok(args);
        }

        // Parse first argument
        args.push(self.parse_logical_or_expression()?);

        // Parse remaining arguments
        while self.lookahead.map(|t| t.kind) == Some(TokenKind::Comma) {
            self.eat(TokenKind::Comma)?;
            args.push(self.parse_logical_or_expression()?);
        }

        Ok(args)
    }

    // NumericLiteral
    //  : NUMBER
    fn parse_numeric_literal(&mut self) -> Result<Expr, &'static str> {
        let token = self.eat(TokenKind::Number)?;
        let lexeme = token.lexeme;

        if let Ok(i) = lexeme.parse::<i64>() {
            Ok(Expr::Literal(Literal::Int(i)))
        } else if let Ok(f) = lexeme.parse::<f64>() {
            Ok(Expr::Literal(Literal::Float(f)))
        } else {
            Err("Invalid number literal")
        }
    }

    // StringLiteral
    //  : STRING
    fn parse_string_literal(&mut self) -> Result<Expr, &'static str> {
        let token = self.eat(TokenKind::String)?;
        Ok(Expr::Literal(Literal::Str(self.intern(token.lexeme))))
    }
}
