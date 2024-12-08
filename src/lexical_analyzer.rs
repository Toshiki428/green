use std::{iter::Peekable, str::Chars, vec};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    FunctionName(String),
    String(String),
    Number(f64),
    Bool(bool),
    AddAndSubOperator(String),
    MulAndDivOperator(String),
    CompareOperator(String),
    LParen,
    RParen,
    Semicolon,
    EOF,
    DocComment(String),
    Comment,
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
/// let tokens = match lexical_analyzer::lex(&content) {
///     Ok(tokens) => tokens,
///     Err(e) => {
///         eprintln!("字句エラー: {}", e);
///         return;
///     }
/// };
/// ```
pub fn lex(text: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();

    while let Some(&char) = chars.peek() {
        match char {
            ' ' | '\n' | '\r' | '\t' => { chars.next(); },  // 無視
            '(' => { tokens.push(Token::LParen); chars.next(); },
            ')' => { tokens.push(Token::RParen); chars.next(); },
            ';' => { tokens.push(Token::Semicolon); chars.next(); },
            '"' => { tokens.push(lex_string(&mut chars)?); },
            '+' => { tokens.push(Token::AddAndSubOperator("+".to_string())); chars.next(); },
            '-' => { tokens.push(Token::AddAndSubOperator("-".to_string())); chars.next(); },
            '*' => { tokens.push(Token::MulAndDivOperator("*".to_string())); chars.next(); },
            '/' => { 
                let slash_string = lex_slash(&mut chars)?;
                match slash_string {
                    Token::Comment => { continue; },  // Commentはtokensに追加しない
                    Token::DocComment(_) => { continue; },  // 一時的にDocCommentも追加しない
                    _ => { tokens.push( slash_string ); },
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
            _ if char.is_alphabetic() => { tokens.push(lex_identifier_or_keyword(&mut chars)?); },
            _ if char.is_numeric() => { tokens.push(lex_number(&mut chars)?); },
            _ => return Err(format!("想定外の文字 {}", char)),
        }
    }
    tokens.push(Token::EOF);
    return Ok(tokens);
}

/// 文字列の字句解析処理
/// 
/// ## Argments
/// 
/// - `chars` - プログラムの文字列
/// 
/// ## Return
/// 
/// - String型のトークン
/// 
/// ## Example
/// 
/// ```
/// tokens.push(lex_string(&mut chars)?);
/// ```
fn lex_string(chars: &mut Peekable<Chars<'_>>) -> Result<Token, String> {
    chars.next();  // 最初の「"」をスキップ
    let mut string = String::new();
    while let Some(&c) = chars.peek() {
        if c == '"' || c == '\n' || c == '\r' { break; }
        string.push(c);
        chars.next();
    }
    // 閉じる「"」をスキップ
    if chars.next() != Some('"') {
        return Err("文字列が閉じられていない".to_string());
    }
    return Ok(Token::String(string));
}

/// スラッシュ記号の字句解析処理
/// 
/// ## Argments
/// 
/// - `chars` - プログラムの文字列
/// 
/// ## Return
/// 
/// - トークン
/// 
/// ## Example
/// 
/// ```
/// let slash_string = lex_slash(&mut chars)?;
/// match slash_string {
///     Token::Comment => { continue; },
///     _ => { tokens.push( slash_string ); },
/// }
/// ```
fn lex_slash(chars: &mut Peekable<Chars<'_>>) -> Result<Token, String> {
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
                    return Ok(Token::DocComment(doc_comment));
                },
                _ => {
                    while let Some(c) = chars.next() {
                        if c == '\n' { break; }
                    }
                    return Ok(Token::Comment);
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
                if chars.peek().is_none() {
                    return Err("コメント中にファイル終了".to_string());
                }
            }
            return Ok(Token::Comment);
        },
        Some(_) => {
            chars.next();
            return Ok(Token::MulAndDivOperator("/".to_string()));
        },
        None => {
            return Err("'/'で入力終了".to_string()); // 入力が尽きた場合
        },
    } 
}

/// 関数、変数、bool値などの字句解析処理
/// 
/// ## Argments
/// 
/// - `chars` - プログラムの文字列
/// 
/// ## Return
/// 
/// - トークン
/// 
/// ## Example
/// 
/// ```
/// tokens.push(lex_identifier_or_keyword(&mut chars)?);
/// ```
fn lex_identifier_or_keyword(chars: &mut Peekable<Chars<'_>>) -> Result<Token, String> {
    let mut string = String::new();
    while let Some(&c) = chars.peek() {
        if !c.is_alphabetic() && !c.is_numeric() { break; }
        string.push(c);
        chars.next();
    }
    match string.as_str() {
        "true" => { Ok(Token::Bool(true)) },
        "false" => { Ok(Token::Bool(false)) },
        _ => { Ok(Token::FunctionName(string)) },
    }
}

/// 数値の字句解析処理
/// 
/// ## Argments
/// 
/// - `chars` - プログラムの文字列
/// 
/// ## Return
/// 
/// - トークン(Float, Int)
/// 
/// ## Example
/// 
/// ```
/// tokens.push(lex_number(&mut chars)?);
/// ```
fn lex_number(chars: &mut Peekable<Chars<'_>>) -> Result<Token, String> {
    let mut number_string = String::new();

    while let Some(&c) = chars.peek() {
        if c.is_numeric() {
            number_string.push(c);
            chars.next();
        } else if c == '.' {
            number_string.push(c);
            chars.next();
        } else if vec![' ', ')', '+', '-', '*', '/', '=', '!'].contains(&c) {
            break;
        } else {
            number_string.push(c);
            chars.next();
            break;
        }
    }

    if let Ok(float_value) = number_string.parse::<f64>() {
        Ok(Token::Number(float_value))
    } else {
        Err(format!("無効なNumber型の数値 {}", number_string))
    }
}