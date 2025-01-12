/// Green言語の型
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Float,
    Int,
    Bool,
    String,
}
impl Type {
    pub fn to_string(&self) -> String {
        match self {
            Self::Int => "int".to_string(),
            Self::Float => "float".to_string(),
            Self::Bool => "bool".to_string(),
            Self::String => "string".to_string(),
        }
    }
}

/// リテラル値
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Float(f64),
    Int(i32),
    Bool(bool),
    String(String),
    Null,
}

impl LiteralValue {
    pub fn to_string(&self) -> String {
        match self {
            Self::Int(i) => i.to_string(),
            Self::Float(f) => f.to_string(),
            Self::String(s) => s.clone(),
            Self::Bool(b) => b.to_string(),
            Self::Null => "Null".to_string(),
        }
    }
}

/// Green言語の値
#[derive(Debug, Clone, PartialEq)]
pub struct GreenValue {
    pub value_type: Type,
    pub value: LiteralValue,
}

impl GreenValue {
    pub fn new(value_type: Type, value: LiteralValue) -> Self {
        Self { value_type, value }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum BlockType {
    /// 関数ブロック
    Function,
    /// 条件分岐ブロック
    Conditional,
    /// ループブロック
    Loop,
    /// グローバルブロック
    Global,
}

impl BlockType {
    pub fn to_string(&self) -> String {
        match self {
            Self::Conditional => "Conditional".to_string(),
            Self::Function => "Function".to_string(),
            Self::Global => "Global".to_string(),
            Self::Loop => "Loop".to_string(),
        }
    }
}
