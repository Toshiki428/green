use super::error_code::ErrorCode;

#[derive(Debug, PartialEq, Clone)]
pub struct ErrorContext {
    pub error_code: ErrorCode,
    pub row: Option<u32>,
    pub col: Option<u32>,
    pub params: Vec<(String, String)>
}
impl ErrorContext {
    pub fn new(error_code: ErrorCode, row: Option<u32>, col:Option<u32>, params: Vec<(&str, &str)>) -> Self {
        let params = params.into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string())) // ここで String に変換
            .collect();
        
        Self { error_code, row, col, params }
    }
}
