#[derive(Debug, Clone, PartialEq)]
pub enum Logical {
    Binary(BinaryLogical),
    Unary(UnaryLogical),
}
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryLogical {
    Not,
}
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryLogical {
    Or,
    And,
    Xor,
}