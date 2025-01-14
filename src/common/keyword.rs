/// 制御構造 (Control Flow) - 条件分岐やループ
#[derive(Debug, Clone, PartialEq)]
pub enum ControlKeyword {
    If,
    Else,
    While,
    For,
    Match,
}
impl ControlKeyword {
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "if" => Some(Self::If),
            "else" => Some(Self::Else),
            "for" => Some(Self::For),
            "while" => Some(Self::While),
            "match" => Some(Self::Match),
            _ => None,
        }
    }
    pub fn to_string(&self) -> String {
        let str = match self {
            Self::If => "if",
            Self::Else => "else",
            Self::For => "for",
            Self::While => "while",
            Self::Match => "match",
        };
        str.to_string()
    }
}

/// 変数や関数の宣言 (Declaration)
#[derive(Debug, Clone, PartialEq)]
pub enum DeclarationKeyword {
    Let,
    Function,
}
impl DeclarationKeyword {
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "let" => Some(Self::Let),
            "function" => Some(Self::Function),
            _ => None,
        }
    }
    pub fn to_string(&self) -> String {
        let str = match self {
            Self::Let => "let",
            Self::Function => "function",
        };
        str.to_string()
    }
}

/// 型 (Type Keywords)
#[derive(Debug, Clone, PartialEq)]
pub enum TypeName {
    Int,
    Float,
    Bool,
    String,
}
impl TypeName {
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "int" => Some(Self::Int),
            "float" => Some(Self::Float),
            "bool" => Some(Self::Bool),
            "string" => Some(Self::String),
            _ => None,
        }
    }
    pub fn to_string(&self) -> String {
        let str = match self {
            Self::Int => "int",
            Self::Float => "float",
            Self::Bool => "bool",
            Self::String => "string",
        };
        str.to_string()
    }
}

/// ループ制御 (Loop Control) - ループ内でのみ使うもの
#[derive(Debug, Clone, PartialEq)]
pub enum LoopControl {
    Break,
    Continue,
}
impl LoopControl {
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "break" => Some(Self::Break),
            "continue" => Some(Self::Continue),
            _ => None,
        }
    }
    pub fn to_string(&self) -> String {
        let str = match self {
            Self::Break => "break",
            Self::Continue => "continue",
        };
        str.to_string()
    }
}

/// 関数制御 (Function Control) - 関数内で使うもの
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionControl {
    Return,
}
impl FunctionControl {
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "return" => Some(Self::Return),
            _ => None,
        }
    }
    pub fn to_string(&self) -> String {
        let str = match self {
            Self::Return => "return",
        };
        str.to_string()
    }
}

/// bool値
#[derive(Debug, Clone, PartialEq)]
pub enum BoolKeyword {
    True,
    False,
}
impl BoolKeyword {
    pub fn from_str(str: &str) -> Option<Self> {
        match str {
            "true" => Some(Self::True),
            "false" => Some(Self::False),
            _ => None,
        }
    }
    pub fn to_string(&self) -> String {
        let str = match self {
            Self::True => "true",
            Self::False => "false",
        };
        str.to_string()
    }
}
