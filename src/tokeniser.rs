use regex::Regex;
use std::{sync::LazyLock, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    LParen,
    RParen,
    OpeningBrace,
    ClosingBrace,
    WhiteSpace,
    Comment,
    Comma,
    Symbol,
    Number,
    String,
    Identifier,
    SimpleAssign,
    Increment,
    Decrement,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Delimeter,
    Let,
    If,
    Else,
    While,
    For,
    In,
    Fn,
    // Comparison operators
    Eq,  // ==
    Neq, // !=
    Gt,  // >
    Lt,  // <
    Gte, // >=
    Lte, // <=
    Eof,
    True,
    False,
    Null,
    And,
    Or,
    BitwiseAnd,
    BitwiseOr,
    // Unary operators
    Bang,  // !
    Tilde, // ~
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TokenKind::LParen         => "(",
            TokenKind::RParen         => ")",
            TokenKind::OpeningBrace   => "{",
            TokenKind::ClosingBrace   => "}",
            TokenKind::WhiteSpace     => "whitespace",
            TokenKind::Comment        => "comment",
            TokenKind::Comma          => ",",
            TokenKind::Symbol         => "symbol",
            TokenKind::Number         => "number",
            TokenKind::String         => "string",
            TokenKind::Identifier     => "identifier",
            TokenKind::SimpleAssign   => "=",
            TokenKind::Increment      => "++",
            TokenKind::Decrement      => "--",
            TokenKind::Plus           => "+",
            TokenKind::Minus          => "-",
            TokenKind::Star           => "*",
            TokenKind::Slash          => "/",
            TokenKind::Percent        => "%",
            TokenKind::Delimeter      => "delimiter",
            TokenKind::Let            => "let",
            TokenKind::If             => "if",
            TokenKind::Else           => "else",
            TokenKind::While          => "where",
            TokenKind::For            => "for",
            TokenKind::In             => "in",
            TokenKind::Fn             => "fn",

            // Comparisons
            TokenKind::Eq             => "==",
            TokenKind::Neq            => "!=",
            TokenKind::Gt             => ">",
            TokenKind::Lt             => "<",
            TokenKind::Gte            => ">=",
            TokenKind::Lte            => "<=",

            TokenKind::Eof            => "EOF",
            TokenKind::True           => "true",
            TokenKind::False          => "false",
            TokenKind::Null           => "null",

            TokenKind::And            => "&&",
            TokenKind::Or             => "||",

            TokenKind::BitwiseAnd     => "&",
            TokenKind::BitwiseOr      => "|",

            TokenKind::Bang           => "!",
            TokenKind::Tilde          => "~",
        };

        write!(f, "{}", s)
    }
}

static TOKEN_PATTERNS: LazyLock<Vec<(TokenKind, Regex)>> = LazyLock::new(|| {
    vec![
        (TokenKind::WhiteSpace, Regex::new(r"^[ \t\n\r]+").unwrap()),
        (
            TokenKind::Comment,
            Regex::new(r"^(?://[^\n]*|/\*[\s\S]*?\*/)").unwrap(),
        ),
        (TokenKind::LParen, Regex::new(r"^\(").unwrap()),
        (TokenKind::RParen, Regex::new(r"^\)").unwrap()),
        (TokenKind::Comma, Regex::new(r"^\,").unwrap()),
        (TokenKind::OpeningBrace, Regex::new(r"^\{").unwrap()),
        (TokenKind::ClosingBrace, Regex::new(r"^\}").unwrap()),
        (
            TokenKind::String,
            Regex::new(r#"^"((?:[^"\\]|\\.)*)""#).unwrap(),
        ),
        // Comparison operators (must come before SimpleAssign)
        (TokenKind::Eq, Regex::new(r"^==").unwrap()),
        (TokenKind::Neq, Regex::new(r"^!=").unwrap()),
        // Unary operators (! must come after !=)
        (TokenKind::Bang, Regex::new(r"^!").unwrap()),
        (TokenKind::Tilde, Regex::new(r"^~").unwrap()),
        (TokenKind::Gte, Regex::new(r"^>=").unwrap()),
        (TokenKind::Lte, Regex::new(r"^<=").unwrap()),
        (TokenKind::Gt, Regex::new(r"^>").unwrap()),
        (TokenKind::Lt, Regex::new(r"^<").unwrap()),
        (TokenKind::SimpleAssign, Regex::new(r"^=").unwrap()),
        (TokenKind::Decrement, Regex::new(r"^--").unwrap()),
        (TokenKind::Increment, Regex::new(r"^\+\+").unwrap()),
        (TokenKind::Plus, Regex::new(r"^\+").unwrap()),
        (TokenKind::Minus, Regex::new(r"^-").unwrap()),
        (TokenKind::Star, Regex::new(r"^\*").unwrap()),
        (TokenKind::Slash, Regex::new(r"^/").unwrap()),
        (TokenKind::Percent, Regex::new(r"^%").unwrap()),
        (TokenKind::And, Regex::new(r"^&&").unwrap()),
        (TokenKind::Or, Regex::new(r"^\|\|").unwrap()),
        (TokenKind::BitwiseAnd, Regex::new(r"^&").unwrap()),
        (TokenKind::BitwiseOr, Regex::new(r"^\|").unwrap()),
        (TokenKind::Number, Regex::new(r"^[0-9]+\.?[0-9]*").unwrap()),
        // Keywords must come before generic Identifier
        (TokenKind::Let, Regex::new(r"^let\b").unwrap()),
        (TokenKind::If, Regex::new(r"^if\b").unwrap()),
        (TokenKind::Else, Regex::new(r"^else\b").unwrap()),
        (TokenKind::While, Regex::new(r"^where\b").unwrap()),
        (TokenKind::For, Regex::new(r"^for\b").unwrap()),
        (TokenKind::In, Regex::new(r"^in\b").unwrap()),
        (TokenKind::Fn, Regex::new(r"^fn\b").unwrap()),
        (TokenKind::True, Regex::new(r"^true\b").unwrap()),
        (TokenKind::False, Regex::new(r"^false\b").unwrap()),
        (TokenKind::Null, Regex::new(r"^null\b").unwrap()),
        (TokenKind::Identifier, Regex::new(r"^[a-zA-Z_]\w*").unwrap()),
        (TokenKind::Delimeter, Regex::new(r"^;").unwrap()),
        (TokenKind::Symbol, Regex::new(r"^[^\s()]+").unwrap()),
    ]
});

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token<'src> {
    pub kind: TokenKind,
    pub lexeme: &'src str,
    pub line: u32,
}

pub struct Tokeniser<'src> {
    program: &'src str,
    pos: usize,
    line: u32,
}

impl<'src> Tokeniser<'src> {
    pub fn new(program: &'src str) -> Self {
        Tokeniser {
            program,
            pos: 0,
            line: 1,
        }
    }

    #[allow(dead_code)]
    pub fn has_more_tokens(&self) -> bool {
        self.program.len() > self.pos
    }

    fn remaining(&self) -> &'src str {
        &self.program[self.pos..]
    }

    fn make_token(&self, kind: TokenKind, lexeme: &'src str) -> Token<'src> {
        Token {
            kind,
            lexeme,
            line: self.line,
        }
    }

    pub fn get_next_token(&mut self) -> Result<Token<'src>, &'static str> {
        let remaining = self.remaining();
        if remaining.is_empty() {
            return Ok(self.make_token(TokenKind::Eof, ""));
        }

        for (kind, regex) in TOKEN_PATTERNS.iter() {
            if let Some(caps) = regex.captures(remaining) {
                let full_match = caps.get(0).unwrap();

                // For strings, use capture group 1 (inner content), otherwise use full match
                let lexeme_match = if *kind == TokenKind::String {
                    caps.get(1).unwrap_or(full_match)
                } else {
                    full_match
                };

                if *kind == TokenKind::WhiteSpace || *kind == TokenKind::Comment {
                    self.line += full_match.as_str().chars().filter(|&c| c == '\n').count() as u32;
                    self.pos += full_match.len();
                    return self.get_next_token();
                }

                let start = self.pos + lexeme_match.start();
                let end = self.pos + lexeme_match.end();
                self.pos += full_match.len();
                return Ok(self.make_token(*kind, &self.program[start..end]));
            }
        }

        // Check for unterminated string
        if remaining.starts_with('"') {
            return Err("Unterminated string");
        }

        Err("Unexpected character")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let mut tokeniser = Tokeniser::new("(1 + 2)");

        let tok = tokeniser.get_next_token().unwrap();
        assert_eq!(tok.kind, TokenKind::LParen);
        assert_eq!(tok.lexeme, "(");

        let tok = tokeniser.get_next_token().unwrap();
        assert_eq!(tok.kind, TokenKind::Number);
        assert_eq!(tok.lexeme, "1");

        let tok = tokeniser.get_next_token().unwrap();
        assert_eq!(tok.kind, TokenKind::Plus);
        assert_eq!(tok.lexeme, "+");

        let tok = tokeniser.get_next_token().unwrap();
        assert_eq!(tok.kind, TokenKind::Number);
        assert_eq!(tok.lexeme, "2");

        let tok = tokeniser.get_next_token().unwrap();
        assert_eq!(tok.kind, TokenKind::RParen);
        assert_eq!(tok.lexeme, ")");

        let tok = tokeniser.get_next_token().unwrap();
        assert_eq!(tok.kind, TokenKind::Eof);
    }

    #[test]
    fn test_string() {
        let mut tokeniser = Tokeniser::new(r#""hello world""#);
        let tok = tokeniser.get_next_token().unwrap();
        assert_eq!(tok.kind, TokenKind::String);
        assert_eq!(tok.lexeme, "hello world");
    }

    #[test]
    fn test_string_with_escapes_raw() {
        let mut tokeniser = Tokeniser::new(r#""hello\nworld""#);
        let tok = tokeniser.get_next_token().unwrap();
        assert_eq!(tok.kind, TokenKind::String);
        assert_eq!(tok.lexeme, r#"hello\nworld"#);
    }

    #[test]
    fn test_comments() {
        let mut tokeniser = Tokeniser::new("// comment\n42");
        let tok = tokeniser.get_next_token().unwrap();
        assert_eq!(tok.kind, TokenKind::Number);
        assert_eq!(tok.lexeme, "42");
        assert_eq!(tok.line, 2);
    }

    #[test]
    fn test_operators() {
        let mut tokeniser = Tokeniser::new("-42 +3.14");

        let tok = tokeniser.get_next_token().unwrap();
        assert_eq!(tok.kind, TokenKind::Minus);
        assert_eq!(tok.lexeme, "-");

        let tok = tokeniser.get_next_token().unwrap();
        assert_eq!(tok.kind, TokenKind::Number);
        assert_eq!(tok.lexeme, "42");

        let tok = tokeniser.get_next_token().unwrap();
        assert_eq!(tok.kind, TokenKind::Plus);
        assert_eq!(tok.lexeme, "+");

        let tok = tokeniser.get_next_token().unwrap();
        assert_eq!(tok.kind, TokenKind::Number);
        assert_eq!(tok.lexeme, "3.14");
    }

    #[test]
    fn test_increment_decrement() {
        let mut tokeniser = Tokeniser::new("++ --");

        let tok = tokeniser.get_next_token().unwrap();
        assert_eq!(tok.kind, TokenKind::Increment);
        assert_eq!(tok.lexeme, "++");

        let tok = tokeniser.get_next_token().unwrap();
        assert_eq!(tok.kind, TokenKind::Decrement);
        assert_eq!(tok.lexeme, "--");
    }

    #[test]
    fn test_line_tracking() {
        let mut tokeniser = Tokeniser::new("a\nb\n\nc");

        let tok = tokeniser.get_next_token().unwrap();
        assert_eq!(tok.lexeme, "a");
        assert_eq!(tok.line, 1);

        let tok = tokeniser.get_next_token().unwrap();
        assert_eq!(tok.lexeme, "b");
        assert_eq!(tok.line, 2);

        let tok = tokeniser.get_next_token().unwrap();
        assert_eq!(tok.lexeme, "c");
        assert_eq!(tok.line, 4);
    }
}
