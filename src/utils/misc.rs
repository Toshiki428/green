use std::{fs::File, io::Read};

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
