use std::{fs::File, io::{Result, Read}};

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
}

/// fileの読み込み
/// 
/// ## Argments
/// 
/// - `file_path` - 読み取りたいファイルのpath
/// 
/// ## Example
/// 
/// ```
/// let content = load_file_content("example.txt")
/// ```
fn load_file_content(file_path: &str) -> Result<String> {
    let mut file = File::open(file_path)?;

    let mut content = String::new();
    file.read_to_string(&mut content)?;

    return Ok(content);
}

pub mod lexical_analyzer {
    #[derive(Debug)]
    pub enum Token {
        Print,
        LParen,
        RParen,
        Semicolon,
        String(String),
        EOF,
    }

    /// 
    /// トークナイズを行う
    /// 
    /// ## Argments
    /// 
    /// - `text` - トークナイズを行う文字列
    /// 
    /// ## Example
    /// 
    /// ```
    /// let token = tokenize(&text)
    /// ```
    pub fn tokenize(text: &str) -> Result<Vec<Token>, String>{
        if text.is_empty() {
            return Err("Input text is empty".to_string());
        }

        let mut tokens = Vec::new();
        let mut chars = text.chars().peekable();

        while let Some(&char) = chars.peek() {
            match char {
                ' ' | '\n' | '\r' => { chars.next(); },  // 無視
                '(' => { tokens.push(Token::LParen); chars.next(); },
                ')' => { tokens.push(Token::RParen); chars.next(); },
                ';' => { tokens.push(Token::Semicolon); chars.next(); },
                '"' => {
                    chars.next();  // 最初の「"」をスキップ
                    let mut string = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == '"' { break; }
                        string.push(c);
                        chars.next();
                    }
                    // 閉じる「"」をスキップ
                    if chars.next().is_none() {
                        return Err("Unclosed string literal".to_string());
                    }
                    tokens.push(Token::String(string));
                },
                _ if char.is_alphabetic() => {
                    let mut function_name = String::new();
                    while let Some(&c) = chars.peek() {
                        if !c.is_alphabetic() { break; }
                        function_name.push(c);
                        chars.next();
                    }
                    if function_name == "print" {
                        tokens.push(Token::Print);
                    } else {
                        return Err(format!("Unknown function: {}", function_name));
                    }
                },
                _ => return Err(format!("Unexpected character: {}", char)),
            }
        }
        tokens.push(Token::EOF);
        return Ok(tokens);
    }
}