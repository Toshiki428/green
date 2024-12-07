#[derive(Debug, Clone)]
pub enum Token {
    Print,
    String(String),
    Int(i32),
    Float(f64),
    Bool(bool),
    AddAndSubOperator(String),
    MulAndDivOperator(String),
    CompareOperator(String),
    LParen,
    RParen,
    Semicolon,
    EOF,
    DocComment(String),
}

/// トークナイズを行う
/// 
/// ## Argments
/// 
/// - `text` - トークナイズを行う文字列
/// 
/// ## Return
/// 
/// - トークン列
/// 
/// ## Example
/// 
/// ```
/// let tokens = match lexical_analyzer::tokenize(&content) {
///     Ok(tokens) => tokens,
///     Err(e) => {
///         eprintln!("Error tokenizing text: {}", e);
///         return;
///     }
/// };
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
            '+' => { tokens.push(Token::AddAndSubOperator("+".to_string())); chars.next(); },
            '-' => { tokens.push(Token::AddAndSubOperator("-".to_string())); chars.next(); },
            '*' => { tokens.push(Token::MulAndDivOperator("*".to_string())); chars.next(); },
            '/' => {
                chars.next();
                match chars.peek() {
                    Some('/') => {
                        chars.next();
                        match chars.peek() {
                            Some('/') => {
                                chars.next();
                                let mut doc_comment = String::new();
                                while let Some(c) = chars.next() {
                                    match c {
                                        '\n' => { break; }
                                        '\r' => {},
                                        _ => { doc_comment.push(c) },
                                    }
                                }
                                // 変数や関数を実数したとき実装
                                // tokens.push(Token::DocComment(doc_comment));
                            },
                            _ => {
                                while let Some(c) = chars.next() {
                                    if c == '\n' { break; }
                                }
                            },
                        }
                    },
                    Some('*') => {
                        chars.next();
                        while let Some(c) = chars.next() {
                            if c == '*' {
                                if let Some('/') = chars.peek() {
                                    chars.next();
                                    break;
                                }
                            }
                        }
                    },
                    Some(_) => {
                        tokens.push(Token::MulAndDivOperator("/".to_string()));
                        chars.next();
                    },
                    None => {
                        return Err("Unexpected end of input".to_string()); // 入力が尽きた場合
                    },
                } 
            },
            '=' => {
                chars.next();
                match chars.peek() {
                    Some('=') => { tokens.push(Token::CompareOperator("==".to_string())); chars.next(); },
                    _ => { return Err("".to_string()); },
                }
            },
            '!' => {
                chars.next();
                match chars.peek() {
                    Some('=') => { tokens.push(Token::CompareOperator("!=".to_string())); chars.next(); },
                    _ => { return Err("".to_string()); }
                }
            }
            _ if char.is_alphabetic() => {
                let mut string = String::new();
                while let Some(&c) = chars.peek() {
                    if !c.is_alphabetic() { break; }
                    string.push(c);
                    chars.next();
                }
                match string.as_str() {
                    "print" => { tokens.push(Token::Print); },
                    "true" => { tokens.push(Token::Bool(true)); },
                    "false" => { tokens.push(Token::Bool(false)); },
                    _ => { return Err(format!("Unknown function: {}", string)); },
                }
            },
            _ if char.is_numeric() => {
                let mut number_string = String::new();
                let mut is_float = false;

                while let Some(&c) = chars.peek() {
                    if c.is_numeric() {
                        number_string.push(c);
                        chars.next();
                    } else if c == '.' {
                        if is_float {
                            return Err("Unexpected '.' in number".to_string());
                        }
                        is_float = true;
                        number_string.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }

                if is_float {
                    if let Ok(float_value) = number_string.parse::<f64>() {
                        tokens.push(Token::Float(float_value));
                    } else {
                        return Err(format!("Invalid float number: {}", number_string));
                    }
                } else {
                    if let Ok(int_value) = number_string.parse::<i32>() {
                        tokens.push(Token::Int(int_value));
                    } else {
                        return Err(format!("Invalid integer number: {}", number_string));
                    }
                }
            },
            _ => return Err(format!("Unexpected character: {}", char)),
        }
    }
    tokens.push(Token::EOF);
    return Ok(tokens);
}