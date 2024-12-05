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
    #[derive(Debug, Clone)]
    pub enum Token {
        Print,
        LParen,
        RParen,
        Semicolon,
        String(String),
        Int(i32),
        Float(f64),
        AddAndSubOperator(String),
        MulAndDivOperator(String),
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
}

mod parser {
    use std::{vec, vec::IntoIter, iter::Peekable};
    use crate::lexical_analyzer::Token;

    #[derive(Debug)]
    pub enum NodeKind {
        Program,
        FunctionCall { name: String },
        Argument,
        String { value: String },
        Expression,
        AddAndSub { operator: String },
        MulAndDiv { operator: String },
        Primary,
        Number,
        Int { value: i32 },
        Float {value: f64 },
    }
    #[derive(Debug)]
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
                NodeKind::Argument => println!("Argument:"),
                NodeKind::String { value } => println!("String: {}", value),
                NodeKind::Expression => println!("Expression:"),
                NodeKind::AddAndSub { operator } => println!("AddAndSub:{}", operator),
                NodeKind::MulAndDiv { operator } => println!("MulAndDiv:{}", operator),
                NodeKind::Primary => println!("Primary:"),
                NodeKind::Number => println!("Number:"),
                NodeKind::Int { value } => println!("Int: {}", value),
                NodeKind::Float { value } => println!("Float: {}", value),
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
                },
                Token::EOF => {
                    tokens.next();
                    break;
                },
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
                },
                _ => Err("Expected '(' after function name".to_string()),
            }
        } else {
            Err("Expected function name".to_string())
        }
    }

    fn parse_argument(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
        match tokens.peek() {
            Some(Token::String(_)) => {
                let string_value = parse_string(tokens)?;
                Ok(Node {
                    kind: NodeKind::Argument,
                    children: vec![string_value],
                })
            },
            Some(Token::Int(_)) | Some(Token::Float(_)) => {
                let expression_value = parse_expression(tokens)?;
                Ok(Node {
                    kind: NodeKind::Argument,
                    children: vec![expression_value],
                })
            },
            _ => Err("Expected argument (string, int, or float)".to_string()),
        }
    }

    fn parse_string(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
        let token = tokens.next();
        if let Some(Token::String(value)) = token {
            Ok(Node {
                kind: NodeKind::String { value: value },
                children: vec![],
            })
        } else {
            Err("".to_string())
        }
    }

    fn parse_expression(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
        let add_and_sub = parse_add_and_sub(tokens)?;
        Ok(Node {
            kind: NodeKind::Expression,
            children: vec![add_and_sub],
        })
    }

    fn parse_add_and_sub(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
        let mut left = parse_mul_and_div(tokens)?;

        while let Some(Token::AddAndSubOperator(operator)) = tokens.peek().cloned() {
            tokens.next();
            let right = parse_mul_and_div(tokens)?;
            left = Node {
                kind: NodeKind::AddAndSub { operator: operator.to_string() },
                children: vec![left, right]
            };
        }
        Ok(left)
    }

    fn parse_mul_and_div(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
        let mut left = parse_primary(tokens)?;

        while let Some(Token::MulAndDivOperator(operator)) = tokens.peek().cloned() {
            tokens.next();
            let right = parse_primary(tokens)?;
            left = Node {
                kind: NodeKind::MulAndDiv { operator: operator.to_string() },
                children: vec![left, right]
            };
        }
        Ok(left)
    }

    fn parse_primary(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
        match tokens.peek(){
            Some(Token::Int(_)) | Some(Token::Float(_)) => {
                let number = parse_number(tokens)?;
                Ok(Node {
                    kind: NodeKind::Primary,
                    children: vec![number],
                })
            },
            Some(Token::LParen) => {
                tokens.next();
                let expr = parse_add_and_sub(tokens)?;
    
                // 閉じカッコの確認
                match tokens.next() {
                    Some(Token::RParen) => Ok(Node {
                        kind: NodeKind::Primary,
                        children: vec![expr],
                    }),
                    _ => Err("Expected ')' after expression".to_string()),
                }
            },
            _ => { Err("Expected primary expression".to_string()) },
        }
    }

    fn parse_number(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
        match tokens.peek(){
            Some(Token::Int(_)) => {
                let int_value = parse_int(tokens)?;
                Ok(Node {
                    kind: NodeKind::Number,
                    children: vec![int_value],
                })
            },
            Some(Token::Float(_)) => {
                let float_value = parse_float(tokens)?;
                Ok(Node {
                    kind: NodeKind::Number,
                    children: vec![float_value],
                })
            },
            _ => { Err("Expected a number".to_string()) },
        }
    }

    fn parse_int(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
        if let Some(Token::Int(value)) = tokens.next() {
            Ok(Node {
                kind: NodeKind::Int { value: value },
                children: vec![],
            })
        } else {
            Err("".to_string())
        }
    }

    fn parse_float(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
        if let Some(Token::Float(value)) = tokens.next() {
            Ok(Node {
                kind: NodeKind::Float { value: value },
                children: vec![],
            })
        } else {
            Err("".to_string())
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
                    print_function(node)
                } else {
                    Err(format!("Unknown function: {}", name))
                }
            },
            _ => Err("Unsupported node type".to_string()),
        }
    }

    fn print_function(node: &Node) -> Result<(), String> {
        if let Some(argument) = node.children.get(0) {
            match &argument.kind {
                NodeKind::Argument => {
                    if let Some(first_child) = argument.children.get(0){
                        match &first_child.kind {
                            NodeKind::String { value } => {
                                println!("{}", value);
                                Ok(())
                            },
                            NodeKind::Expression => {
                                if let Some(expression_child) = first_child.children.get(0){
                                    if let Some(result) = evaluate_expression(expression_child) {
                                        println!("{}", result);
                                        Ok(())
                                    } else {
                                        Err("Failed to evaluate the numerical expression".to_string())
                                    }
                                } else {
                                    Err("Err".to_string())
                                }
                            },
                            _ => Err("Unsupported argument type in Argument node".to_string()),
                        }
                    } else {
                        Err("Argument node is empty".to_string())
                    }
                },
                _ => Err("Invalid argument to function 'print'".to_string()),
            }
        } else {
            Err("Missing argument to function 'print'".to_string())
        }
    }

    fn evaluate_expression(node: &Node) -> Option<f64> {
        match &node.kind {
            NodeKind::Float { value } => Some( *value as f64 ),
            NodeKind::Int { value } => Some( *value as f64 ),
            NodeKind::Number => {
                let number = node.children.get(0)?;
                let value = evaluate_expression(number)?;
                Some(value)
            },
            NodeKind::Primary => {
                let number = node.children.get(0)?;
                let value = evaluate_expression(number)?;
                Some(value)
            },
            NodeKind::MulAndDiv { operator } => {
                let left = node.children.get(0)?;
                let left_value = evaluate_expression(left)?;
                match operator.as_str() {
                    "*" => {
                        let right = node.children.get(1)?;
                        let right_value = evaluate_expression(right)?;
                        Some(left_value * right_value)
                    },
                    "/" => {
                        let right = node.children.get(1)?;
                        let right_value = evaluate_expression(right)?;
                        Some(left_value / right_value)
                    },
                    "" => Some(left_value),
                    _ => None,
                }
            },
            NodeKind::AddAndSub { operator } => {
                let left = node.children.get(0)?;
                let left_value = evaluate_expression(left)?;
                match operator.as_str() {
                    "+" => {
                        let right = node.children.get(1)?;
                        let right_value = evaluate_expression(right)?;
                        Some(left_value + right_value)
                    },
                    "-" => {
                        let right = node.children.get(1)?;
                        let right_value = evaluate_expression(right)?;
                        Some(left_value - right_value)
                    },
                    "" => Some(left_value),
                    _ => None,
                }
            },
            _ => None,
        }
    }
}