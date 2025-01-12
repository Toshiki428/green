#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    If,
    Else,
    While,
    For,
    Match,
    Let,
    Function,
    Int,
    Float,
    Bool,
    String,
    Return,
}

impl Keyword {
    pub fn to_string(&self) -> String {
        let str = match self {
            Self::If => "if",
            Self::Else => "else",
            Self::While => "while",
            Self::For => "for",
            Self::Match => "match",
            Self::Let => "let",
            Self::Function => "function",
            Self::Int => "int",
            Self::Float => "float",
            Self::Bool => "bool",
            Self::String => "string",
            Self::Return => "return",
        };
        str.to_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BoolKeyword {
    True,
    False,
}
