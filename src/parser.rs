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
    Compare { operator: String },
    Bool { value: bool },
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
            NodeKind::Compare { operator } => println!("Compare: {}", operator),
            NodeKind::Bool { value } => println!("Bool: {}", value),
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
        Some(Token::Bool(_)) => {
            let bool_value = parse_bool(tokens)?;
            Ok(Node {
                kind: NodeKind::Argument,
                children: vec![bool_value],
            })
        }
        _ => Err("Expected argument (string, int, float or bool)".to_string()),
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
    let add_and_sub = parse_compare(tokens)?;
    Ok(Node {
        kind: NodeKind::Expression,
        children: vec![add_and_sub],
    })
}

fn parse_compare(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
    let mut left = parse_add_and_sub(tokens)?;

    if let Some(Token::CompareOperator(operator)) = tokens.peek().cloned() {
        tokens.next();
        let right = parse_add_and_sub(tokens)?;
        left = Node {
            kind: NodeKind::Compare { operator: operator.to_string() },
            children: vec![left, right]
        }
    }
    Ok(left)
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
        Some(Token::String(_)) => {
            let string = parse_string(tokens)?;
            Ok(Node {
                kind: NodeKind::Primary,
                children: vec![string]
            })
        }
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

fn parse_bool(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
    if let Some(Token::Bool(value)) = tokens.next() {
        Ok(Node {
            kind: NodeKind::Bool { value: value },
            children: vec![],
        })
    } else {
        Err("".to_string())
    }
}