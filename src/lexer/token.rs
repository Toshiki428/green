use crate::{
    lexer::keyword::{BoolKeyword, Keyword},
    common::{
        operator::{Arithmetic, Comparison, Logical},
        types::Type, 
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(String),
    BoolLiteral(BoolKeyword),
    ArithmeticOperator(Arithmetic),
    CompareOperator(Comparison),
    LogicalOperator(Logical),
    Keyword(Keyword),
    VariableType(Type),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Equal,
    Colon,
    Semicolon,
    Comma,
    Dot,
    EOF,
    DocComment(String),
    Comment,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub row: u32,
    pub col: u32,
}

impl TokenKind {
    pub fn to_string(&self) -> String {
        let token_str = match self {
            Self::ArithmeticOperator(operator) => &operator.to_string(),
            Self::BoolLiteral(operator) => &format!("{:?}", operator),
            Self::Colon => ":",
            Self::Comma => ",",
            Self::Comment => "",
            Self::CompareOperator(operator) => &operator.to_string(),
            Self::DocComment(_) => "",
            Self::Dot => ".",
            Self::EOF => "EOF",
            Self::Equal => "==",
            Self::Identifier(string) => string,
            Self::Keyword(keyword) => &format!("{:?}", keyword),
            Self::LBrace => "{",
            Self::LParen => "(",
            Self::LogicalOperator(operator) => &operator.to_string(),
            Self::NumberLiteral(string) => string,
            Self::RBrace => "}",
            Self::RParen => ")",
            Self::Semicolon => ";",
            Self::StringLiteral(string) => string,
            Self::VariableType(varialbe_type) => &varialbe_type.to_string(),
        };
        token_str.to_string()
    }
}
