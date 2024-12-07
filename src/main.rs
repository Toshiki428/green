use std::{fs::File, io::{Result, Read}};

mod lexical_analyzer;
mod parser;
mod interpreter;

fn main() {
    let file_path = "main.grn";
    let content = match load_file_content(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return;
        }
    };

    let tokens = match lexical_analyzer::tokenize(&content) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Error tokenizing text: {}", e);
            return;
        }
    };
    println!("Tokens: {:?}", tokens);

    let ast = match parser::create_ast(tokens){
        Ok(node) => node,
        Err(e) => {
            eprintln!("Error parsing text: {}", e);
            return;
        }
    };
    ast.print(0);

    if let Err(e) = interpreter::execute_ast(&ast) {
        eprintln!("Error execute: {}", e);
        return;
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
