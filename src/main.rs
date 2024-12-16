use std::{env, fs::File, io::{Result, Read}};

mod cli_arg_parse;
mod lexical_analyzer;
mod parser;
mod interpreter;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = match cli_arg_parse::Config::new(&args){
        Ok(config) => config,
        Err(e) => {
            eprintln!("コマンドライン解析エラー: {}", e);
            return;
        }
    };
    
    let content = match load_file_content(&config.file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    };

    let tokens = match lexical_analyzer::lex(&content) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("字句エラー: {}", e);
            return;
        }
    };

    let ast = match parser::parse(tokens){
        Ok(node) => node,
        Err(e) => {
            eprintln!("構文エラー: {}", e);
            return;
        }
    };

    if config.option == cli_arg_parse::RunMode::Execute {
        if let Err(e) = interpreter::execute(&ast) {
            eprintln!("実行エラー: {}", e);
            return;
        }
    }
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
fn load_file_content(file_path: &str) -> Result<String> {
    let mut file = File::open(file_path)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    return Ok(content);
}
