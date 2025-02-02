use crate::common::{
    keyword::*,
    operator::{Arithmetic, Comparison, Logical},
};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // 識別子やリテラル
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(String),
    BoolLiteral(BoolKeyword),

    // 演算子や記号
    ArithmeticOperator(Arithmetic),
    CompareOperator(Comparison),
    LogicalOperator(Logical),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Equal,
    Colon,
    Semicolon,
    Comma,
    Dot,
    RArrow,

    // キーワード
    ControlKeyword(ControlKeyword),
    DeclarationKeyword(DeclarationKeyword),
    TypeName(TypeName),
    LoopControl(LoopControl),
    FunctionControl(FunctionControl),
    CoroutineControl(CoroutineControl),

    // 終端
    EOF,

    // その他
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
            Self::Identifier(string) => string,
            Self::BoolLiteral(operator) => &operator.to_string(),
            Self::NumberLiteral(string) => string,
            Self::StringLiteral(string) => string,

            Self::ArithmeticOperator(operator) => &operator.to_string(),
            Self::CompareOperator(operator) => &operator.to_string(),
            Self::LogicalOperator(operator) => &operator.to_string(),
            Self::Colon => ":",
            Self::Comma => ",",
            Self::LBrace => "{",
            Self::LParen => "(",
            Self::Equal => "=",
            Self::RBrace => "}",
            Self::RParen => ")",
            Self::Semicolon => ";",
            Self::Dot => ".",
            Self::RArrow => "->",
            
            Self::ControlKeyword(keyword) => &keyword.to_string(),
            Self::TypeName(type_name) => &type_name.to_string(),
            Self::DeclarationKeyword(keyword) => &keyword.to_string(),
            Self::LoopControl(keyword) => &keyword.to_string(),
            Self::FunctionControl(keyword) => &keyword.to_string(),
            Self::CoroutineControl(keyword) => &keyword.to_string(),

            Self::EOF => "EOF",
            
            Self::Comment => "",
            Self::DocComment(_) => "",
            
        };
        token_str.to_string()
    }
}
