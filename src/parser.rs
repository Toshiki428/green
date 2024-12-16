use std::{iter::Peekable, vec::IntoIter};
use crate::lexical_analyzer::{Token, TokenKind};

#[derive(Debug, PartialEq)]
pub enum NodeKind {
    Program,
    FunctionCall { name: String },
    Argument,
    Expression,
    Compare { operator: String },
    AddAndSub { operator: String },
    MulAndDiv { operator: String },
    Unary { operator: String },
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
            NodeKind::Unary { operator } => println!("Unary: {}", operator),
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

struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self{
            tokens: tokens.into_iter().peekable(),
        }
    }

    /// ルートの構文解析
    fn parse_program(&mut self) -> Result<Node, String> {
        let mut children = Vec::new();

        while let Some(token) = self.tokens.peek() {
            match token.kind {
                TokenKind::FunctionName(_) => {
                    children.push(self.parse_function_call()?);
                },
                TokenKind::EOF => {
                    self.tokens.next();
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
    /// ## Return
    /// 
    /// - Node
    fn parse_function_call(&mut self) -> Result<Node, String> {
        let token = self.tokens.next().ok_or("トークンが不足")?;
        let function_name = if let TokenKind::FunctionName(name) = token.kind {
            name
        } else {
            return Err("想定外の関数呼び出し".to_string());
        };

        let token = self.tokens.next().ok_or("'('が不足")?;
        if token.kind != TokenKind::LParen {
            return Err("関数名の後には'('が必要".to_string())
        }

        let argument = self.parse_argument()?;

        let token = self.tokens.next().ok_or("')'が不足")?;
        if token.kind != TokenKind::RParen {
            return Err("引数の後には')'が必要".to_string());
        }

        let token = self.tokens.next().ok_or("';'が不足")?;
        if token.kind != TokenKind::Semicolon {
            return Err("関数呼び出しの後には';'が必要".to_string());
        }

        Ok(Node {
            kind: NodeKind::FunctionCall { name: function_name },
            children: vec![argument],
        })
    }

    /// 引数の構文解析
    fn parse_argument(&mut self) -> Result<Node, String> {
        let token = self.tokens.peek().ok_or("トークンが空")?;
        match token.kind {
            TokenKind::String(_) | TokenKind::Number(_) | TokenKind::AddAndSubOperator(_) | TokenKind::LParen => {
                let expression = self.parse_expression()?;
                Ok(Node {
                    kind: NodeKind::Argument,
                    children: vec![expression],
                })
            },
            TokenKind::Bool(_) => {
                let bool_value = self.parse_bool()?;
                Ok(Node {
                    kind: NodeKind::Argument,
                    children: vec![bool_value],
                })
            },
            _ => Err("引数は(string, number, bool)のみ".to_string()),
        }
    }

    /// 式の構文解析
    fn parse_expression(&mut self) -> Result<Node, String> {
        let add_and_sub = self.parse_compare()?;
        Ok(Node {
            kind: NodeKind::Expression,
            children: vec![add_and_sub],
        })
    }

    /// 比較式の構文解析
    fn parse_compare(&mut self) -> Result<Node, String> {
        let left = self.parse_value()?;
        let token = self.tokens.peek().ok_or("トークンが空")?;
        let operator = if let TokenKind::CompareOperator(value) = token.kind.clone() {
            self.tokens.next();
            value
        } else {
            return Ok(Node {
                kind: NodeKind::Compare { operator: "".to_string() },
                children: vec![left]
            })
        };

        let right = self.parse_value()?;
        return Ok(Node {
            kind: NodeKind::Compare { operator },
            children: vec![left, right]
        });
    }

    /// 値の構文解析
    fn parse_value(&mut self) -> Result<Node, String> {
        let token = self.tokens.peek().ok_or("トークンが空")?;
        match token.kind {
            TokenKind::Number(_) | TokenKind::AddAndSubOperator(_) | TokenKind::LParen => {
                let node = self.parse_add_and_sub()?;
                Ok(node)
            },
            TokenKind::String(_) => {
                let node = self.parse_string()?;
                Ok(node)
            },
            _ => { Err(format!("想定外のToken(value):{:?}", token)) },
        }
    }

    /// 足し算、引き算の構文解析
    fn parse_add_and_sub(&mut self) -> Result<Node, String> {
        let mut left = self.parse_mul_and_div()?;
        while let Some(TokenKind::AddAndSubOperator(operator)) = self.tokens.peek().map(|t| &t.kind) {
            let operator = operator.clone();
            self.tokens.next();
            let right = self.parse_mul_and_div()?;
            left = Node {
                kind: NodeKind::AddAndSub { operator },
                children: vec![left, right]
            };
        }
        Ok(left)
    }

    /// 掛け算、引き算の構文解析
    fn parse_mul_and_div(&mut self) -> Result<Node, String> {
        let mut left = self.parse_unary()?;
        while let Some(TokenKind::MulAndDivOperator(operator)) = self.tokens.peek().map(|t| &t.kind) {
            let operator = operator.clone();
            self.tokens.next();
            let right = self.parse_unary()?;
            left = Node {
                kind: NodeKind::MulAndDiv { operator },
                children: vec![left, right]
            };
        }
        Ok(left)
    }

    /// 単項演算子の構文解析
    fn parse_unary(&mut self) -> Result<Node, String> {
        let token = self.tokens.peek().ok_or("空のノード")?;
        match token.kind.clone() {
            TokenKind::Number(_) | TokenKind::LParen => {
                let number = self.parse_primary()?;
                Ok(Node {
                    kind: NodeKind::Unary { operator: "+".to_string() },
                    children: vec![number],
                })
            },
            TokenKind::AddAndSubOperator(operator) => {
                self.tokens.next();
                let number = self.parse_primary()?;
                Ok(Node {
                    kind: NodeKind::Unary { operator: operator.to_string() },
                    children: vec![number],
                })
            },
            _ => { Err(format!("想定外のToken(unary):{:?}", token)) },
        }
    }

    /// 数値、計算式の'()'の構文解析
    fn parse_primary(&mut self) -> Result<Node, String> {
        let token = self.tokens.peek().ok_or("トークンが空")?;
        match token.kind{
            TokenKind::Number(_) => {
                let number = self.parse_number()?;
                Ok(Node {
                    kind: NodeKind::Primary,
                    children: vec![number],
                })
            },
            TokenKind::LParen => {
                self.tokens.next();
                let expr = self.parse_add_and_sub()?;

                // 閉じカッコの確認
                match self.tokens.next().ok_or("トークンが空")?.kind {
                    TokenKind::RParen => Ok(Node {
                        kind: NodeKind::Primary,
                        children: vec![expr],
                    }),
                    _ => Err("計算式の')'が必要".to_string()),
                }
            },
            _ => { Err(format!("想定外のToken(primary):{:?}", token)) },
        }
    }

    /// String型の構文解析
    fn parse_string(&mut self) -> Result<Node, String> {
        let token = self.tokens.next().ok_or("トークンが空")?;
        if let TokenKind::String(value) = token.kind {
            Ok(Node {
                kind: NodeKind::String { value: value },
                children: vec![],
            })
        } else {
            Err("想定外のString型".to_string())
        }
    }

    /// Number型の構文解析
    fn parse_number(&mut self) -> Result<Node, String> {
        let token = self.tokens.next().ok_or("空のトークン")?;
        if let TokenKind::Number(value) = token.kind {
            Ok(Node {
                kind: NodeKind::Number { value },
                children: vec![],
            })
        } else {
            Err("想定外のNumber型".to_string())
        }
    }

    /// bool型の構文解析
    fn parse_bool(&mut self) -> Result<Node, String> {
        let token = self.tokens.next().ok_or("空のトークン")?;
        if let TokenKind::Bool(value) = token.kind {
            Ok(Node {
                kind: NodeKind::Bool { value: value },
                children: vec![],
            })
        } else {
            Err("想定外のbool型".to_string())
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
/// let ast = match parser::parse(tokens){
///     Ok(node) => node,
///     Err(e) => {
///         eprintln!("構文エラー: {}", e);
///         return;
///     }
/// };
/// ```
pub fn parse(tokens: Vec<Token>) -> Result<Node, String> {
    let mut parser = Parser::new(tokens);
    let node = parser.parse_program()?;
    Ok(node)
}