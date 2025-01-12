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

impl Logical {
    pub fn to_string(&self) -> String {
        let operator = match self {
            Self::Binary(binary_logical) => {
                match binary_logical {
                    BinaryLogical::And => "and",
                    BinaryLogical::Or => "or",
                    BinaryLogical::Xor => "xor",
                }
            },
            Self::Unary(unary_logical) => {
                match unary_logical {
                    UnaryLogical::Not => "not",
                }
            }
        };
        operator.to_string()
    }
}

impl Arithmetic {
    pub fn to_string(&self) -> String {
        let operator = match self {
            Arithmetic::Binary(binary_arithmetic) => {
                match binary_arithmetic {
                    BinaryArithmetic::Add => "+",
                    BinaryArithmetic::Subtract => "-",
                    BinaryArithmetic::Multiply => "*",
                    BinaryArithmetic::Divide => "/",
                }
            },
            Arithmetic::Unary(unary_arithmetic) => {
                match unary_arithmetic {
                    UnaryArithmetic::Plus => "+",
                    UnaryArithmetic::Minus => "-",
                }
            },
        };
        operator.to_string()
    }
}

impl Comparison {
    pub fn to_string(&self) -> String {
        let operator = match self {
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::Greater => ">",
            Self::Less => "<",
            Self::GreaterEqual => ">=",
            Self::LessEqual => "<=",
        };
        operator.to_string()
    }
}