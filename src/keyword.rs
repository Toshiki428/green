#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    If,
    Else,
    While,
    For,
    Match,
    Let,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BoolValue {
    True,
    False,
}
