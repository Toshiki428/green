use serde_json::Value;
use std::{fs::File, io::Read};

/// エラーメッセージの取得
pub fn get_error_message(code: &str, params: &[(&str, &str)]) -> Result<String, String> {
    let error_message = load_error_messages()?;
    if let Some(template) = error_message.get(code).and_then(|v| v.as_str()) {
        let mut message = template.to_string();
        for &(key, value) in params {
            message = message.replace(&format!("{{{}}}", key), value);
        }
        Ok(message)
    } else {
        Err(format!("不正なエラーコード: {}", code))
    }
}

/// 場所を指定したエラーメッセージの取得
pub fn get_error_message_with_location(code: &str, row: u32, col: u32, params: &[(&str, &str)]) -> Result<String, String> {
    let mut message = get_error_message(code, params)?;
    message = message.replace(&"{row}", &row.to_string());
    message = message.replace(&"{col}", &col.to_string());
    Ok(message)
}

/// エラーメッセージのロード
fn load_error_messages() -> Result<Value, String> {
    let err_msg_str = load_file_content("assets/error_message.json")
        .map_err(|e| format!("JSONファイル読み込みエラー: {}", e))?;

    let error_message: Value = serde_json::from_str(&err_msg_str)
        .map_err(|e| format!("JSONパースエラー: {}", e))?;

    Ok(error_message)
}

/// fileの読み込み
/// 
/// ## Argments
/// 
/// - `file_path` - 読み取りたいファイルのpath
/// 
/// ## Return
/// 
/// - 読み取ったファイルの中身の文字列
/// 
/// ## Example
/// 
/// ```
/// let content = match load_file_content(file_path) {
///     Ok(content) => content,
///     Err(e) => {
///         eprintln!("Error reading file: {}", e);
///         return;
///     }
/// };
/// ```
pub fn load_file_content(file_path: &str) -> Result<String, std::io::Error> {
    let mut file = File::open(file_path)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    return Ok(content);
}
