use std::{iter::Peekable, str::Chars, vec};

use crate::{utils, operator::{Logical, UnaryLogical, BinaryLogical, Arithmetic, UnaryArithmetic, BinaryArithmetic, Comparison}};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(String),
    BoolLiteral(String),
    ArithmeticOperator(Arithmetic),
    CompareOperator(Comparison),
    LogicalOperator(Logical),
    LParen,
    RParen,
    Equal,
    Semicolon,
    EOF,
    DocComment(String),
    Comment,
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub row: u32,
    pub col: u32,
}

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    tokens: Vec<Token>,
    row: u32,
    col: u32,
}

impl<'a> Lexer<'a> {
    fn new(text: &'a str) -> Self {
        let chars = text.chars().peekable();
        Self {
            chars,
            tokens: Vec::new(),
            row: 1,
            col: 1,
        }
    }

    fn tokenize(mut self) -> Result<Vec<Token>, String> {
        while let Some(&char) = self.chars.peek() {
            match char {
                ' ' | '\n' | '\r' | '\t' =>  self.next_char(),
                '(' => {self.push_token(TokenKind::LParen); self.next_char();},
                ')' => {self.push_token(TokenKind::RParen); self.next_char();},
                ';' => {self.push_token(TokenKind::Semicolon); self.next_char();},
                '+' => {self.push_token(TokenKind::ArithmeticOperator(Arithmetic::Unary(UnaryArithmetic::Plus))); self.next_char();},
                '-' => {self.push_token(TokenKind::ArithmeticOperator(Arithmetic::Unary(UnaryArithmetic::Minus))); self.next_char();},
                '*' => {self.push_token(TokenKind::ArithmeticOperator(Arithmetic::Binary(BinaryArithmetic::Multiply))); self.next_char();},
                '/' => self.lex_slash()?,
                '"' => self.lex_string()?,
                '=' => self.lex_equal()?,
                '!' => self.lex_exclamation()?,
                '<' | '>' => self.lex_angle()?,
                _ if char.is_alphabetic() => self.lex_identifier()?,
                _ if char.is_numeric() => self.lex_number()?,
                _ => return Err(utils::get_error_message_with_location("LEX002", self.row, self.col, &[("char", &char.to_string())])?),
            }
        }
        self.push_token(TokenKind::EOF);
        return Ok(self.tokens);
    }

    /// 文字列の字句解析処理
    /// 
    /// ## Example
    /// 
    /// ```
    /// lexer.lex_string()?;
    /// ```
    fn lex_string(&mut self) -> Result<(), String> {
        self.next_char();  // 最初の「"」をスキップ
        let mut string = String::new();
        while let Some(&c) = self.chars.peek() {
            if c == '"' || c == '\n' || c == '\r' { break; }
            string.push(c);
            self.next_char();
        }

        if self.chars.peek() != Some(&'"') {
            return Err(utils::get_error_message_with_location("LEX003", self.row, self.col, &[])?);
        }
        self.next_char();   // 閉じる「"」をスキップ
        self.push_token(TokenKind::StringLiteral(string));
        Ok(())
    }

    /// スラッシュ記号の字句解析処理
    fn lex_slash(&mut self) -> Result<(), String> {
        let start_row = self.row;
        let start_col = self.col;
        self.next_char();       // 最初の`/`をスキップ
        let token_kind: TokenKind;

        let c = self.chars.peek();
        match c {
            Some(&'/') => {
                self.next_char();
                match self.chars.peek() {
                    Some('/') => {
                        self.next_char();
                        let mut doc_comment = String::new();
                        while let Some(&c) = self.chars.peek() {
                            self.next_char();
                            match c {
                                '\n' => { break; }
                                '\r' => {},
                                _ => { doc_comment.push(c) },
                            }
                        }
                        // 変数や関数を実数したとき改めて実装
                        token_kind = TokenKind::DocComment(doc_comment);
                    },
                    _ => {
                        while let Some(&c) = self.chars.peek() {
                            self.next_char();
                            if c == '\n' { break; }
                        }
                        token_kind = TokenKind::Comment;
                    },
                }
            },
            Some(&'*') => {
                self.next_char();
                while let Some(&c) = self.chars.peek() {
                    self.next_char();
                    if c == '*' {
                        if let Some('/') = self.chars.peek() {
                            self.next_char();
                            break;
                        }
                    }
                }
                if self.chars.peek().is_none() {
                    return Err(utils::get_error_message_with_location("LEX004", self.row, self.col, &[])?);
                }
                token_kind = TokenKind::Comment;
            },
            _ => {
                token_kind = TokenKind::ArithmeticOperator(Arithmetic::Binary(BinaryArithmetic::Divide));
            },
        } 

        match token_kind {
            TokenKind::Comment => {},  // Commentはtokensに追加しない
            TokenKind::DocComment(_) => {},  // 一時的にDocCommentも追加しない
            TokenKind::ArithmeticOperator(_) => {self.push_token_with_location(token_kind, start_row, start_col);},
            _ => { self.push_token(token_kind); },
        }
        Ok(())
    }

    /// イコール記号の字句解析処理
    fn lex_equal(&mut self) -> Result<(), String> {
        let mut operator = "=".to_string();
        self.next_char();
        if let Some(&c) = self.chars.peek() {
            operator.push(c);
            match operator.as_str() {
                "==" => { self.push_token(TokenKind::CompareOperator(Comparison::Equal)); self.next_char(); },
                _ => { self.push_token(TokenKind::Equal); },
                
            }
        }
        Ok(())
    }

    /// ビックリマークの字句解析処理
    fn lex_exclamation(&mut self) -> Result<(), String> {
        let mut operator = "!".to_string();
        self.next_char();
        if let Some(&c) = self.chars.peek() {
            operator.push(c);
            match operator.as_str() {
                "!=" => { self.push_token(TokenKind::CompareOperator(Comparison::NotEqual)); self.next_char(); },
                _ => { return Err(utils::get_error_message_with_location("LEX005", self.row, self.col, &[("operator", &operator)])?); },
            }
        }
        Ok(())
    }

    /// `<`と`>`の字句解析
    fn lex_angle(&mut self) -> Result<(), String> {
        let start_col = self.col;
        let mut operator = match self.chars.peek() {
            Some(&c) => c.to_string(),
            _ => return Err(utils::get_error_message("ALL", &[])?),
        };
        self.next_char();
        while let Some(&c) = self.chars.peek() {
            match c {
                '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' | ' ' | '\r' |'\n' => {
                    break;
                },
                _ => {
                    operator.push(c);
                    self.next_char();
                },
            }
        }
        match operator.as_str() {
            "<=" => self.push_token(TokenKind::CompareOperator(Comparison::LessEqual)),
            "<" => self.push_token(TokenKind::CompareOperator(Comparison::Less)),
            ">=" => self.push_token(TokenKind::CompareOperator(Comparison::GreaterEqual)),
            ">" => self.push_token(TokenKind::CompareOperator(Comparison::Greater)),
            _ => { return Err(utils::get_error_message_with_location("LEX005", self.row, start_col, &[("operator", &operator)])?); }
        }
        Ok(())
    }

    /// 関数、変数、bool値などの字句解析処理
    fn lex_identifier(&mut self) -> Result<(), String> {
        let start_col = self.col;
        let mut string = String::new();
        while let Some(&c) = self.chars.peek() {
            if !c.is_alphabetic() && !c.is_numeric() { break; }
            string.push(c);
            self.next_char();
        }
        match string.as_str() {
            "true" | "false" => self.push_token_with_location(TokenKind::BoolLiteral(string), self.row, start_col),
            "or" => self.push_token_with_location(TokenKind::LogicalOperator(Logical::Binary(BinaryLogical::Or)), self.row, self.col),
            "and" => self.push_token_with_location(TokenKind::LogicalOperator(Logical::Binary(BinaryLogical::And)), self.row, self.col),
            "xor" => self.push_token_with_location(TokenKind::LogicalOperator(Logical::Binary(BinaryLogical::Xor)), self.row, self.col),
            "not" => self.push_token_with_location(TokenKind::LogicalOperator(Logical::Unary(UnaryLogical::Not)), self.row, self.col),
            _ => self.push_token_with_location(TokenKind::Identifier(string), self.row, start_col),
        }

        Ok(())
    }

    /// 数値の字句解析
    fn lex_number(&mut self) -> Result<(), String> {
        let start_col = self.col;
        let mut number_string = String::new();

        while let Some(&c) = self.chars.peek() {
            if c.is_numeric() || c == '.' {
                number_string.push(c);
                self.next_char();
            } else if vec![' ', ')', ';', '+', '-', '*', '/', '=', '!', '\r', '\n'].contains(&c) {
                break;
            } else {
                number_string.push(c);
                self.next_char();
                break;
            }
        }

        if let Err(_) = number_string.parse::<f64>() {
            return Err(utils::get_error_message_with_location("LEX006", self.row, start_col, &[("number", &number_string)])?);
        }
    
        self.push_token_with_location(TokenKind::NumberLiteral(number_string), self.row, start_col);
        Ok(())
    }

    /// tokenの追加
    /// 
    /// ## Argument
    /// 
    /// - `kind` - 追加するトークンの種類
    /// 
    /// ## example
    /// 
    /// ```
    /// push_token(TokenKind::LParen)
    /// ```
    fn push_token(&mut self, kind: TokenKind) {
        self.tokens.push(Token { kind, row: self.row, col: self.col });
    }

    /// ポジションを指定してtokenを追加する
    fn push_token_with_location(&mut self, kind: TokenKind, row: u32, col: u32) {
        self.tokens.push(Token { kind, row, col });
    }

    /// 次の文字へ進む
    /// 文字の行数や列数のカウントも行う
    /// 
    /// ## Example
    /// 
    /// ```
    /// lexer.next_char();
    /// ```
    fn next_char(&mut self) {
        match self.chars.next() {
            Some('\n') => {
                self.row += 1;
                self.col = 1;
            }, 
            _ => {
                self.col += 1;
            },
        }
    }
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
    let lexer = Lexer::new(text);
    lexer.tokenize()
}
