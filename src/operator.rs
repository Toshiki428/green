#[derive(Debug, Clone, PartialEq)]
pub enum Logical {
    Unary(UnaryLogical),
    Binary(BinaryLogical),
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

#[derive(Debug, Clone, PartialEq)]
pub enum Arithmetic {
    Unary(UnaryArithmetic),
    Binary(BinaryArithmetic),
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryArithmetic {
    Plus, // 正号
    Minus, // 負号
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryArithmetic {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Comparison {
    Equal,         // ==
    NotEqual,      // !=
    Greater,       // >
    Less,          // <
    GreaterEqual,  // >=
    LessEqual,     // <=
}
