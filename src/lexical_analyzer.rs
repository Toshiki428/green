use std::{iter::Peekable, str::Chars, vec};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
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
                '+' => {self.push_token(TokenKind::AddAndSubOperator("+".to_string())); self.next_char();},
                '-' => {self.push_token(TokenKind::AddAndSubOperator("-".to_string())); self.next_char();},
                '*' => {self.push_token(TokenKind::MulAndDivOperator("*".to_string())); self.next_char();},
                '/' => self.lex_slash()?,
                '"' => self.lex_string()?,
                '=' => self.lex_equal()?,
                '!' => self.lex_exclamation()?,
                '<' => self.lex_left_angle()?,
                '>' => self.lex_right_angle()?,
                _ if char.is_alphabetic() => self.lex_identifier()?,
                _ if char.is_numeric() => self.lex_number()?,
                _ => return Err(format!("想定外の文字 {}", char)),
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
            return Err("文字列が閉じられていない".to_string());
        }
        self.next_char();   // 閉じる「"」をスキップ
        self.push_token(TokenKind::String(string));
        Ok(())
    }

    /// スラッシュ記号の字句解析処理
    fn lex_slash(&mut self) -> Result<(), String> {
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
                    if self.chars.peek().is_none() {
                        return Err("コメント中にファイル終了".to_string());
                    }
                }
                token_kind = TokenKind::Comment;
            },
            Some(_) => {
                self.next_char();
                token_kind = TokenKind::MulAndDivOperator("/".to_string());
            },
            None => {
                return Err("'/'で入力終了".to_string()); // 入力が尽きた場合
            },
        } 

        match token_kind {
            TokenKind::Comment => {},  // Commentはtokensに追加しない
            TokenKind::DocComment(_) => {},  // 一時的にDocCommentも追加しない
            _ => { self.push_token(token_kind); },
        }
        Ok(())
    }

    /// イコール記号の字句解析処理
    fn lex_equal(&mut self) -> Result<(), String> {
        self.next_char();
        match self.chars.peek() {
            Some('=') => { self.push_token(TokenKind::CompareOperator("==".to_string())); self.next_char(); },
            _ => { return Err(format!("定義されていない演算子: {:?} {} {}", self.chars.peek(), self.row, self.col)); },
        }
        Ok(())
    }

    /// ビックリマークの字句解析処理
    fn lex_exclamation(&mut self) -> Result<(), String> {
        self.next_char();
        match self.chars.peek() {
            Some('=') => { self.push_token(TokenKind::CompareOperator("!=".to_string())); self.next_char(); },
            _ => { return Err(format!("定義されていない演算子: {:?}", self.chars.peek())); }
        }
        Ok(())
    }

    /// `<`の字句解析
    fn lex_left_angle(&mut self) -> Result<(), String> {
        self.next_char();
        match self.chars.peek() {
            Some('=') => { self.push_token(TokenKind::CompareOperator("<=".to_string())); self.next_char(); },
            Some(_) => { self.push_token(TokenKind::CompareOperator("<".to_string())); },
            _ => { return Err(format!("定義されていない演算子: {:?}", self.chars.peek())); }
        }
        Ok(())
    }

    /// `>`の字句解析
    fn lex_right_angle(&mut self) -> Result<(), String> {
        self.next_char();
        match self.chars.peek() {
            Some('=') => { self.push_token(TokenKind::CompareOperator(">=".to_string())); self.next_char(); },
            Some(_) => { self.push_token(TokenKind::CompareOperator(">".to_string())); },
            _ => { return Err(format!("定義されていない演算子: {:?}", self.chars.peek())); }
        }
        Ok(())
    }

    /// 関数、変数、bool値などの字句解析処理
    fn lex_identifier(&mut self) -> Result<(), String> {
        let mut string = String::new();
        while let Some(&c) = self.chars.peek() {
            if !c.is_alphabetic() && !c.is_numeric() { break; }
            string.push(c);
            self.next_char();
        }
        match string.as_str() {
            "true" => self.push_token(TokenKind::Bool(true)),
            "false" => self.push_token(TokenKind::Bool(false)),
            _ => self.push_token(TokenKind::FunctionName(string)),
        }

        Ok(())
    }

    /// 数値の字句解析
    fn lex_number(&mut self) -> Result<(), String> {
        let mut number_string = String::new();

        while let Some(&c) = self.chars.peek() {
            if c.is_numeric() || c == '.' {
                number_string.push(c);
                self.next_char();
            } else if vec![' ', ')', '+', '-', '*', '/', '=', '!'].contains(&c) {
                break;
            } else {
                number_string.push(c);
                self.next_char();
                break;
            }
        }
    
        if let Ok(value) = number_string.parse::<f64>() {
            self.push_token(TokenKind::Number(value))
        } else {
            return Err(format!("無効なNumber型の数値 {}", number_string));
        }

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

    /// 次の文字へ進む
    /// 文字の行数や列数のカウントも行う
    /// 
    /// ## Returns
    /// 
    /// - `Option<char>` - 次の文字
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
