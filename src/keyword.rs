#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    If,
    Else,
    While,
    For,
    Match,
    Let,
    Function,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BoolValue {
    True,
    False,
}
