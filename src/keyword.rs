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

#[derive(Debug, Clone, PartialEq)]
pub enum BoolKeyword {
    True,
    False,
}
