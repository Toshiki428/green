use std::{vec, vec::IntoIter, iter::Peekable};
use crate::lexical_analyzer::Token;

#[derive(Debug, PartialEq)]
pub enum NodeKind {
    Program,
    FunctionCall { name: String },
    Argument,
    Expression,
    Compare { operator: String },
    AddAndSub { operator: String },
    MulAndDiv { operator: String },
    Primary,
    String { value: String },
    Number {value: f64 },
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
            NodeKind::Expression => println!("Expression:"),
            NodeKind::Compare { operator } => println!("Compare: {}", operator),
            NodeKind::AddAndSub { operator } => println!("AddAndSub:{}", operator),
            NodeKind::MulAndDiv { operator } => println!("MulAndDiv:{}", operator),
            NodeKind::Primary => println!("Primary:"),
            NodeKind::String { value } => println!("String: {}", value),
            NodeKind::Number { value } => println!("Number: {}", value),
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
/// - 構文解析の結果のASTのルートNode
/// 
/// ## Example
/// 
/// ```
/// let ast = match parser::create_ast(tokens){
///     Ok(node) => node,
///     Err(e) => {
///         eprintln!("構文エラー: {}", e);
///         return;
///     }
/// };
/// ```
pub fn create_ast(tokens: Vec<Token>) -> Result<Node, String> {
    let mut tokens = tokens.into_iter().peekable();
    let node = parse_program(&mut tokens)?;
    Ok(node)
}


/// ルートの構文解析
/// 
/// ## Argments
/// 
/// - `tokens` - トークン列
/// 
/// ## Return
/// 
/// - Node
/// 
/// ## Example
/// 
/// ```
/// let node = parse_program(&mut tokens)?;
/// ```
fn parse_program(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
    let mut children = Vec::new();

    while let Some(token) = tokens.peek() {
        match token {
            Token::FunctionName(_) => {
                children.push(parse_function_call(tokens)?);
            },
            Token::EOF => {
                tokens.next();
                break;
            },
            _ => return Err(format!("想定外のToken(program): {:?}", token)),
        }
    }

    Ok(Node { 
        kind: NodeKind::Program, 
        children: children 
    })
}

/// FunctionCallの構文解析
/// 
/// ## Argments
/// 
/// - `tokens` - トークン列
/// 
/// ## Return
/// 
/// - Node
/// 
/// ## Example
/// 
/// ```
/// let node = parse_function_call(tokens)?;
/// ```
fn parse_function_call(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
    if let Some(Token::FunctionName(function_name)) = tokens.next(){
        if tokens.next() != Some(Token::LParen) {
            return Err("関数名の後には'('が必要".to_string());
        }

        let argument = parse_argument(tokens)?;

        if tokens.next() != Some(Token::RParen) {
            return Err("引数の後には')'が必要".to_string());
        }

        if tokens.next() != Some(Token::Semicolon) {
            return Err("関数呼び出しの後には';'が必要".to_string());
        }

        Ok(Node {
            kind: NodeKind::FunctionCall { name: function_name },
            children: vec![argument],
        })
    } else {
        Err("想定外の関数呼び出し".to_string())
    }
}

/// 引数の構文解析
/// 
/// ## Argments
/// 
/// - `tokens` - トークン列
/// 
/// ## Return
/// 
/// - Node
/// 
/// ## Example
/// 
/// ```
/// let node = parse_argument(tokens)?;
/// ```
fn parse_argument(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
    match tokens.peek() {
        Some(Token::String(_)) | Some(Token::Number(_)) => {
            let expression = parse_expression(tokens)?;
            Ok(Node {
                kind: NodeKind::Argument,
                children: vec![expression],
            })
        },
        Some(Token::Bool(_)) => {
            let bool_value = parse_bool(tokens)?;
            Ok(Node {
                kind: NodeKind::Argument,
                children: vec![bool_value],
            })
        }
        _ => Err("引数は(string, number, bool)のみ".to_string()),
    }
}

/// 式の構文解析
/// 
/// ## Argments
/// 
/// - `tokens` - トークン列
/// 
/// ## Return
/// 
/// - Node
/// 
/// ## Example
/// 
/// ```
/// let node = parse_expression(tokens)?;
/// ```
fn parse_expression(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
    let add_and_sub = parse_compare(tokens)?;
    Ok(Node {
        kind: NodeKind::Expression,
        children: vec![add_and_sub],
    })
}

/// 比較式の構文解析
/// 
/// ## Argments
/// 
/// - `tokens` - トークン列
/// 
/// ## Return
/// 
/// - Node
/// 
/// ## Example
/// 
/// ```
/// let node = parse_compare(tokens)?;
/// ```
fn parse_compare(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
    let left = parse_value(tokens)?;

    if let Some(Token::CompareOperator(operator)) = tokens.peek().cloned() {
        tokens.next();
        let right = parse_value(tokens)?;
        return Ok(Node {
            kind: NodeKind::Compare { operator: operator },
            children: vec![left, right]
        });
    }
    Ok(Node {
        kind: NodeKind::Compare { operator: "".to_string() },
        children: vec![left]
    })
}

/// 値の構文解析
/// 
/// ## Argments
/// 
/// - `tokens` - トークン列
/// 
/// ## Return
/// 
/// - Node
/// 
/// ## Example
/// 
/// ```
/// let node = parse_value(tokens)?;
/// ```
fn parse_value(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
    let next_token = tokens.peek();
    match next_token {
        Some(Token::Number(_)) => {
            let node = parse_add_and_sub(tokens)?;
            Ok(node)
        },
        Some(Token::String(_)) => {
            let node = parse_string(tokens)?;
            Ok(node)
        },
        _ => { Err(format!("想定外のToken(value):{:?}", next_token)) },
    }
}

/// 足し算、引き算の構文解析
/// 
/// ## Argments
/// 
/// - `tokens` - トークン列
/// 
/// ## Return
/// 
/// - Node
/// 
/// ## Example
/// 
/// ```
/// let node = parse_add_and_sub(tokens)?;
/// ```
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

/// 掛け算、引き算の構文解析
/// 
/// ## Argments
/// 
/// - `tokens` - トークン列
/// 
/// ## Return
/// 
/// - Node
/// 
/// ## Example
/// 
/// ```
/// let node = parse_mul_and_div(tokens)?;
/// ```
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

/// 数値、計算式の'()'の構文解析
/// 
/// ## Argments
/// 
/// - `tokens` - トークン列
/// 
/// ## Return
/// 
/// - Node
/// 
/// ## Example
/// 
/// ```
/// let node = parse_primary(tokens)?;
/// ```
fn parse_primary(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
    let next_token = tokens.peek();
    match next_token{
        Some(Token::Number(_)) => {
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
                _ => Err("計算式の')'が必要".to_string()),
            }
        },
        _ => { Err(format!("想定外のToken(primary):{:?}", next_token)) },
    }
}

/// String型の構文解析
/// 
/// ## Argments
/// 
/// - `tokens` - トークン列
/// 
/// ## Return
/// 
/// - Node
/// 
/// ## Example
/// 
/// ```
/// let node = parse_string(tokens)?;
/// ```
fn parse_string(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
    if let Some(Token::String(value)) = tokens.next() {
        Ok(Node {
            kind: NodeKind::String { value: value },
            children: vec![],
        })
    } else {
        Err("想定外のString型".to_string())
    }
}

/// Number型の構文解析
/// 
/// ## Argments
/// 
/// - `tokens` - トークン列
/// 
/// ## Return
/// 
/// - Node
/// 
/// ## Example
/// 
/// ```
/// let node = parse_number(tokens)?;
/// ```
fn parse_number(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
    if let Some(Token::Number(value)) = tokens.next() {
        Ok(Node {
            kind: NodeKind::Number { value: value },
            children: vec![],
        })
    } else {
        Err("想定外のNumber型".to_string())
    }
}

/// bool型の構文解析
/// 
/// ## Argments
/// 
/// - `tokens` - トークン列
/// 
/// ## Return
/// 
/// - Node
/// 
/// ## Example
/// 
/// ```
/// let node = parse_bool(tokens)?;
/// ```
fn parse_bool(tokens: &mut Peekable<IntoIter<Token>>) -> Result<Node, String> {
    if let Some(Token::Bool(value)) = tokens.next() {
        Ok(Node {
            kind: NodeKind::Bool { value: value },
            children: vec![],
        })
    } else {
        Err("想定外のbool型".to_string())
    }
}
