#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    If,
    While,
    For,
    Match,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BoolValue {
    True,
    False,
}
