use serde_json::Value;
use crate::error::error_code::ErrorCode;
use std::sync::OnceLock;

static ERROR_MESSAGE: OnceLock<ErrorMessage> = OnceLock::new();
const JSON_DATA: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/error_message.json"));

pub struct ErrorMessage {
    pub error_message: Value,
}

impl ErrorMessage {
    pub fn global() -> &'static Self {
        ERROR_MESSAGE.get_or_init(|| {
            let error_message = serde_json::from_str(JSON_DATA)
                .unwrap_or_else(|e| panic!("JSONパースエラー: {}", e));
            Self { error_message }
        })
    }
    
    /// エラーメッセージの取得
    pub fn get_error_message(&self, code: &ErrorCode, params: &[(&str, &str)]) -> Result<String, String> {
        let code = code.to_string();
        if let Some(template) = self.error_message.get(&code).and_then(|v| v.as_str()) {
            let mut message = template.to_string();
            for &(key, value) in params {
                message = message.replace(&format!("{{{}}}", key), value);
            }
            Ok(message)
        } else {
            Err(format!("不正なエラーコード: {}", &code))
        }
    }
    
    /// 場所を指定したエラーメッセージの取得
    pub fn get_error_message_with_location(&self, code: &ErrorCode, row: u32, col: u32, params: &[(&str, &str)]) -> Result<String, String> {
        let mut message = self.get_error_message(code, params)?;
        message = message.replace(&"{row}", &row.to_string());
        message = message.replace(&"{col}", &col.to_string());
        Ok(message)
    }
}
