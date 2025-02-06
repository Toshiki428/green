use serde_json::Value;
use std::sync::OnceLock;
use super::error_context::ErrorContext;

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
    pub fn get_error_message(&self, error: ErrorContext) -> Result<String, String> {
        let code = error.error_code.to_string();
        if let Some(template) = self.error_message.get(&code).and_then(|v| v.as_str()) {
            let mut message = template.to_string();
            for (key, value) in &error.params {
                message = message.replace(&format!("{{{}}}", key), value);
            }

            if let Some(row) = &error.row {
                message = message.replace(&"{row}", &row.to_string())
            }
            if let Some(col) = &error.col {
                message = message.replace(&"{col}", &col.to_string())
            }
            
            Ok(message)
        } else {
            Err(format!("不正なエラーコード: {}", &code))
        }
    }
}
