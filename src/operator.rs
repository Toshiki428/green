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
    pub fn as_str(&self) -> &str {
        match self {
            Self::Binary(binary_logical) => {
                match binary_logical {
                    BinaryLogical::And => return "and",
                    BinaryLogical::Or => return "or",
                    BinaryLogical::Xor => return "xor",
                }
            },
            Self::Unary(unary_logical) => {
                match unary_logical {
                    UnaryLogical::Not => return "not",
                }
            }
        }
    }
}

impl Arithmetic {
    pub fn as_str(&self) -> &str {
        match self {
            Arithmetic::Binary(binary_arithmetic) => {
                match binary_arithmetic {
                    BinaryArithmetic::Add => return "+",
                    BinaryArithmetic::Subtract => return "-",
                    BinaryArithmetic::Multiply => return "*",
                    BinaryArithmetic::Divide => return "/",
                }
            },
            Arithmetic::Unary(unary_arithmetic) => {
                match unary_arithmetic {
                    UnaryArithmetic::Plus => return "+",
                    UnaryArithmetic::Minus => return "-",
                }
            },
        }
    }
}

impl Comparison {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::Greater => ">",
            Self::Less => "<",
            Self::GreaterEqual => ">=",
            Self::LessEqual => "<=",
        }
    }
}