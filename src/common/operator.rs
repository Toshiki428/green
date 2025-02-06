/// 論理演算
#[derive(Debug, Clone, PartialEq)]
pub enum Logical {
    Unary(UnaryLogical),
    Binary(BinaryLogical),
}
impl Logical {
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "and" => Some(Self::Binary(BinaryLogical::And)),
            "or" => Some(Self::Binary(BinaryLogical::Or)),
            "xor" => Some(Self::Binary(BinaryLogical::Xor)),
            "not" => Some(Self::Unary(UnaryLogical::Not)),
            _ => None,
        }
    }
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

/// 四則演算
#[derive(Debug, Clone, PartialEq)]
pub enum Arithmetic {
    Plus,
    Minus,
    Multiply,
    Divide,
}
impl Arithmetic {
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "+" => Some(Self::Plus),
            "-" => Some(Self::Minus),
            "*" => Some(Self::Multiply),
            "/" => Some(Self::Divide),
            _ => None,
        }
    }
    pub fn to_string(&self) -> String {
        let operator = match self {
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
        };
        operator.to_string()
    }
}

/// 比較演算
#[derive(Debug, Clone, PartialEq)]
pub enum Comparison {
    Equal,         // ==
    NotEqual,      // !=
    Greater,       // >
    Less,          // <
    GreaterEqual,  // >=
    LessEqual,     // <=
}
impl Comparison {
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "==" => Some(Self::Equal),
            "!=" => Some(Self::NotEqual),
            ">" => Some(Self::Greater),
            "<" => Some(Self::Less),
            ">=" => Some(Self::GreaterEqual),
            "<=" => Some(Self::LessEqual),
            _ => None,
        }
    }
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