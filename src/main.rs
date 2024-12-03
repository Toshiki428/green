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

pub mod lexical_analyzer {
    #[derive(Debug)]
    pub enum Token {
        Print,
        LParen,
        RParen,
        Semicolon,
        String(String),
        Int(i32),
        Float(f64),
        DocComment(String),
        EOF,
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
                                }
                                _ => {
                                    while let Some(c) = chars.next() {
                                        if c == '\n' { break; }
                                    }
                                }
                            }

                        }
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
                        }
                        Some(other) => {
                            return Err(format!("Unknown syntax: /{}", other));
                        }
                        None => {
                            return Err("Unexpected end of input".to_string()); // 入力が尽きた場合
                        }
                    } 
                }
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
                        }else {
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
                }
                _ => return Err(format!("Unexpected character: {}", char)),
            }
        }
        tokens.push(Token::EOF);
        return Ok(tokens);
    }
}

mod parser {
    use std::{vec, vec::IntoIter, iter::Peekable};
    use crate::lexical_analyzer::Token;

    pub enum NodeKind {
        Program,
        FunctionCall { name: String },
        Argument{ value: String },
    }

    pub struct Node {
        pub kind: NodeKind,
        pub children: Vec<Node>,
    }

    impl Node {
        /// デバッグ用のprint文
        pub fn print(&self, depth: usize) {
            for _ in 0..depth {
                print!("  ");
            }
            match &self.kind {
                NodeKind::Program => println!("Program"),
                NodeKind::FunctionCall { name } => println!("FunctionCall: {}", name),
                NodeKind::Argument { value } => println!("Argument: {}", value),
            }
            for child in &self.children {
                child.print(depth + 1);
            }
        }
    }

    /// 構文解析を行う
    /// 
    /// ## Argments
    /// 
    /// - `tokens` - トークン列
    /// 
    /// ## Return
    /// 
    /// - 構文解析の結果のAST
    /// 
    /// ## Example
    /// 
    /// ```
    /// let ast = match parser::create_ast(tokens){
    ///     Ok(node) => node,
    ///     Err(e) => {
    ///         eprintln!("Error parsing text: {}", e);
    ///         return;
    ///     }
    /// };
    /// ```
    pub fn create_ast(tokens: Vec<Token>) -> Result<Node, String> {
        let mut tokens = tokens.into_iter().peekable();
        let node = parse_program(&mut tokens)?;
        Ok(node)
    }

    fn parse_program(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
        let mut children = Vec::new();

        while let Some(token) = tokens.peek() {
            match token {
                Token::Print => {
                    children.push(parse_function_call(tokens)?);
                }
                Token::EOF => {
                    tokens.next();
                    break;
                }
                _ => return Err(format!("Unexpected token in program: {:?}", token)),
            }
        }

        Ok(Node { 
            kind: NodeKind::Program, 
            children: children 
        })
    }

    fn parse_function_call(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
        if let Some(Token::Print) = tokens.next() {
            match tokens.next() {
                Some(Token::LParen) => {
                    let argument = parse_argument(tokens)?;

                    if let Some(Token::RParen) = tokens.next() {
                        if let Some(Token::Semicolon) = tokens.next() {
                            Ok(Node {
                                kind: NodeKind::FunctionCall { name: "print".to_string() },
                                children: vec![argument],
                            })
                        } else {
                            Err("Expected ';' after function call".to_string())
                        }
                    } else {
                        Err("Expected ')' after arguments".to_string())
                    }
                }
                _ => Err("Expected '(' after function name".to_string()),
            }
        } else {
            Err("Expected function name".to_string())
        }
    }

    fn parse_argument(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
        match tokens.next() {
            Some(Token::String(value)) => Ok(Node {
                kind: NodeKind::Argument { value },
                children: vec![],
            }),
            Some(Token::Int(value)) => Ok(Node {
                kind: NodeKind::Argument { value: value.to_string() },
                children: vec![],
            }),
            Some(Token::Float(value)) => Ok(Node {
                kind: NodeKind::Argument { value: value.to_string() },
                children: vec![],
            }),
            _ => Err("Expected argument (string, int, or float)".to_string())
        }
    }
}

mod interpreter {
    use crate::parser::{Node, NodeKind};

    /// プログラムの実行
    /// 
    /// ## Argments
    /// 
    /// - `node` - ASTのノード
    /// 
    /// ## Return
    /// 
    /// - 実行結果
    /// 
    /// ## Example
    /// 
    /// ```
    /// if let Err(e) = interpreter::execute_ast(&ast) { 
    ///     eprintln!("Error execute: {}", e);
    ///     return;
    /// }
    /// ```
    pub fn execute_ast(node: &Node) -> Result<(), String> {
        match &node.kind {
            NodeKind::Program => {
                for child in &node.children {
                    execute_ast(child)?;
                }
                Ok(())
            },
            NodeKind::FunctionCall { name } => {
                if name == "print" {
                    if let Some(argument) = node.children.get(0) {
                        match &argument.kind {
                            NodeKind::Argument { value } => {
                                println!("{}", value);
                                Ok(())
                            }
                            _ => Err("Invalid argument to function 'print'".to_string())
                        }
                    } else {
                        Err("Missing argument to function 'print'".to_string())
                    }
                } else {
                    Err(format!("Unknown function: {}", name))
                }
            },
            _ => Err("Unsupported node type".to_string())
        }
    }
}