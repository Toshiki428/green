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
        eprintln!("Error parsing text: {}", e);
        return;
    }
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

    pub fn create_ast(tokens: Vec<Token>) -> Result<Node, String> {
        let mut tokens = tokens.into_iter().peekable();
        match parse_program(&mut tokens) {
            Ok(node) => Ok(node),
            Err(e) => Err(e)
        }
    }

    fn parse_program(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
        let mut children = Vec::new();

        while let Some(token) = tokens.peek() {
            match token {
                Token::Print => {
                    children.push(
                        match parse_function_call(tokens) {
                            Ok(node) => node,
                            Err(e) => return Err(e)
                        }
                    );
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
                    let argument = match parse_argument(tokens) {
                        Ok(node) => node,
                        Err(e) => return Err(e),
                    };

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
        if let Some(Token::String(value)) = tokens.next() {
            Ok(Node {
                kind: NodeKind::Argument { value },
                children: vec![],
            })
        } else {
            Err("Expected argument string".to_string())
        }
    }
}

mod interpreter {
    use crate::parser::{Node, NodeKind};

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