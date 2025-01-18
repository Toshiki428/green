use std::{iter::Peekable, str::Chars};
use crate::{
    common::{
        keyword::*, operator::*
    }, error::{
        error_code::ErrorCode, error_context::ErrorContext
    }, lexer::token::{Token, TokenKind}
};

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    tokens: Vec<Token>,
    row: u32,
    col: u32,
    errors: Vec<ErrorContext>,
}

impl<'a> Lexer<'a> {
    fn new(text: &'a str) -> Self {
        let chars = text.chars().peekable();
        Self {
            chars,
            tokens: Vec::new(),
            row: 1,
            col: 1,
            errors: Vec::new(),
        }
    }

    fn tokenize(&mut self) -> (Vec<Token>, Vec<ErrorContext>) {
        while let Some(&char) = self.chars.peek() {
            match char {
                ' ' | '\n' | '\r' | '\t' =>  self.next_char(),
                '(' => {self.push_token(TokenKind::LParen); self.next_char();},
                ')' => {self.push_token(TokenKind::RParen); self.next_char();},
                '{' => {self.push_token(TokenKind::LBrace); self.next_char();},
                '}' => {self.push_token(TokenKind::RBrace); self.next_char();},
                ':' => {self.push_token(TokenKind::Colon); self.next_char();}
                ';' => {self.push_token(TokenKind::Semicolon); self.next_char();},
                ',' => {self.push_token(TokenKind::Comma); self.next_char();},
                '.' => {self.push_token(TokenKind::Dot); self.next_char();}
                '+' | '-' | '*' => {
                    match Arithmetic::from_str(&char.to_string()) {
                        Some(operator) => self.push_token(TokenKind::ArithmeticOperator(operator)),
                        _ => unreachable!(),
                    }
                    self.next_char();
                },
                '/' => {
                    match self.lex_slash() {
                        Ok(_) => {},
                        Err(e) => self.errors.push(e),
                    }
                },
                '"' => {
                    match self.lex_string() {
                        Ok(_) => {},
                        Err(e) => self.errors.push(e),
                    }
                },
                '=' => {
                    match self.lex_equal() {
                        Ok(_) => {},
                        Err(e) => self.errors.push(e),
                    }
                },
                '!' => {
                    match self.lex_exclamation() {
                        Ok(_) => {},
                        Err(e) => self.errors.push(e),
                    }
                },
                '<' | '>' => {
                    match self.lex_angle() {
                        Ok(_) => {},
                        Err(e) => self.errors.push(e),
                    }
                },
                _ if char.is_alphabetic() => {
                    match self.lex_identifier() {
                        Ok(_) => {},
                        Err(e) => self.errors.push(e),
                    }
                },
                _ if char.is_numeric() => {
                    match self.lex_number() {
                        Ok(_) => {},
                        Err(e) => self.errors.push(e),
                    }
                },
                _ => {
                    self.errors.push(ErrorContext::new(
                        ErrorCode::Lex002,
                        self.row, self.col,
                        vec![("char", &char.to_string())]
                    ));
                    self.next_char();
                },
            }
        }
        self.push_token(TokenKind::EOF);
        return (self.tokens.clone(), self.errors.clone());
    }

    /// 文字列の字句解析処理
    /// 
    /// ## Example
    /// 
    /// ```
    /// lexer.lex_string()?;
    /// ```
    fn lex_string(&mut self) -> Result<(), ErrorContext> {
        self.next_char();  // 最初の「"」をスキップ
        let mut string = String::new();
        while let Some(&c) = self.chars.peek() {
            if c == '"' || c == '\n' || c == '\r' { break; }
            string.push(c);
            self.next_char();
        }

        if self.peek_char()? != '"' {
            return Err(
                ErrorContext::new(
                    ErrorCode::Lex003,
                    self.row, self.col,
                    vec![],
                )
            );
        }
        self.next_char();   // 閉じる「"」をスキップ
        self.push_token(TokenKind::StringLiteral(string));
        Ok(())
    }

    /// スラッシュ記号の字句解析処理
    fn lex_slash(&mut self) -> Result<(), ErrorContext> {
        let start_row = self.row;
        let start_col = self.col;
        self.next_char();       // 最初の`/`をスキップ
        let token_kind: TokenKind;

        let symbol = match self.peek_char()? {
            '/' => "//",
            '*' => "/*",
            _ => "/",
        };
        match symbol {
            "//" => {
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
            "/*" => {
                self.next_char();
                loop {
                    let c = self.peek_char()?;
                    self.next_char();
                    if c == '*' {
                        if self.peek_char()? == '/' {
                            self.next_char();
                            break;
                        }
                    }
                }
                token_kind = TokenKind::Comment;
            },
            _ => {
                match Arithmetic::from_str("/") {
                    Some(operator) => token_kind = TokenKind::ArithmeticOperator(operator),
                    None => unreachable!(),
                }
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
    fn lex_equal(&mut self) -> Result<(), ErrorContext> {
        self.next_char();
        let operator = match self.peek_char()? {
            '=' => "==",
            _ => "=",
        };

        match operator {
            "==" => {
                match Comparison::from_str(operator) {
                    Some(op) => self.push_token(TokenKind::CompareOperator(op)),
                    None => unreachable!(),
                }
                self.next_char();
            },
            _ => { self.push_token(TokenKind::Equal); },
            
        }
        Ok(())
    }

    /// ビックリマークの字句解析処理
    fn lex_exclamation(&mut self) -> Result<(), ErrorContext> {
        self.next_char();
        let operator = match self.peek_char()? {
            '=' => "!=",
            _ => "!",
        };

        match operator {
            "!=" => {
                match Comparison::from_str(operator) {
                    Some(op) => self.push_token(TokenKind::CompareOperator(op)),
                    None => unreachable!(),
                }
                self.next_char();
            },
            _ => return Err(
                ErrorContext::new(
                    ErrorCode::Lex005,
                    self.row, self.col,
                    vec![("operator", &operator)],
                )
            ),
        }
        Ok(())
    }

    /// `<`と`>`の字句解析
    fn lex_angle(&mut self) -> Result<(), ErrorContext> {
        let start_col = self.col;
        let mut operator = self.peek_char()?.to_string();
        self.next_char();

        loop {
            let c = self.peek_char()?;
            match c {
                '=' => {
                    operator.push(c);
                    self.next_char();
                },
                _ => {
                    break;
                },
            }
        }

        match operator.as_str() {
            "<=" | "<" | ">=" | ">" => {
                match Comparison::from_str(&operator) {
                    Some(op) => self.push_token(TokenKind::CompareOperator(op)),
                    None => unreachable!(),
                }
            },
            _ => return Err(ErrorContext::new(
                ErrorCode::Lex005,
                self.row, start_col,
                vec![("operator", &operator)],
            )),
        }
        Ok(())
    }

    /// 関数、変数、bool値などの字句解析処理
    fn lex_identifier(&mut self) -> Result<(), ErrorContext> {
        let start_col = self.col;
        let mut string = String::new();
        loop {
            let c = self.peek_char()?;
            
            if !c.is_alphabetic() && !c.is_numeric() && c != '_' { break; }
            string.push(c);
            self.next_char();
        }
        match string.as_str() {
            "true" | "false" => {
                match BoolKeyword::from_str(&string) {
                    Some(keyword) => self.push_token_with_location(TokenKind::BoolLiteral(keyword), self.row, start_col),
                    None => unreachable!(),
                }
            },
            "or" | "and" | "xor" | "not" => {
                match Logical::from_str(&string) {
                    Some(op) => self.push_token_with_location(TokenKind::LogicalOperator(op), self.row, start_col),
                    None => unreachable!(),
                }
            },
            "if" | "else" | "for" | "while" | "match" => {
                match ControlKeyword::from_str(&string) {
                    Some(keyword) => self.push_token_with_location(TokenKind::ControlKeyword(keyword), self.row, start_col),
                    None => unreachable!(),
                }
            },
            "let" | "function" => {
                match DeclarationKeyword::from_str(&string) {
                    Some(keyword) => self.push_token_with_location(TokenKind::DeclarationKeyword(keyword), self.row, start_col),
                    None => unreachable!(),
                }
            },
            "int" | "float" | "bool" | "string" => {
                match TypeName::from_str(&string) {
                    Some(keyword) => self.push_token_with_location(TokenKind::TypeName(keyword), self.row, start_col),
                    None => unreachable!(),
                }
            },
            "return" => {
                match FunctionControl::from_str(&string) {
                    Some(keyword) => self.push_token_with_location(TokenKind::FunctionControl(keyword), self.row, start_col),
                    None => unreachable!(),
                }
            },
            "break" | "continue" => {
                match LoopControl::from_str(&string) {
                    Some(keyword) => self.push_token_with_location(TokenKind::LoopControl(keyword), self.row, start_col),
                    None => unreachable!(),
                }
            }
            _ => self.push_token_with_location(TokenKind::Identifier(string), self.row, start_col),
        }

        Ok(())
    }

    /// 数値の字句解析
    fn lex_number(&mut self) -> Result<(), ErrorContext> {
        let start_col = self.col;
        let mut number_string = String::new();

        while let Some(&c) = self.chars.peek() {
            if c.is_numeric() {
                number_string.push(c);
                self.next_char();
            } else {
                break;
            }
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

    fn peek_char(&mut self) -> Result<char, ErrorContext> {
        match self.chars.peek() {
            Some(c) => Ok(c.clone()),
            None => Err(ErrorContext::new(
                ErrorCode::Lex004,
                self.row, self.col,
                vec![],
            ))
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
pub fn lex(text: &str) -> (Vec<Token>, Vec<ErrorContext>) {
    let mut lexer = Lexer::new(text);
    lexer.tokenize()
}
