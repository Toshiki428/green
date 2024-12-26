use std::{iter::Peekable, vec::IntoIter};
use crate::{lexical_analyzer::{Token, TokenKind}, utils::{self, get_error_message_with_location}};

#[derive(Debug, PartialEq)]
pub enum LiteralValue {
    Int(i32),
    Float(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug, PartialEq)]
pub enum NodeKind {
    Program,
    FunctionCall { name: String },
    VariableDeclaration { name: String },
    Argument,
    Variable { name: String },
    Compare { operator: String },
    AddAndSub { operator: String },
    MulAndDiv { operator: String },
    Unary { operator: String },
    Literal(LiteralValue),
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
            NodeKind::VariableDeclaration { name } => println!("VariableDeclaration: {}", name),
            NodeKind::Variable { name } => println!("Variable: {}", name),
            NodeKind::Compare { operator } => println!("Compare: {}", operator),
            NodeKind::AddAndSub { operator } => println!("AddAndSub:{}", operator),
            NodeKind::MulAndDiv { operator } => println!("MulAndDiv:{}", operator),
            NodeKind::Unary { operator } => println!("Unary: {}", operator),
            NodeKind::Literal ( literal ) => {
                match literal {
                    LiteralValue::Int(value) => println!("Int: {}", value),
                    LiteralValue::Float(value) => println!("Float: {}", value),
                    LiteralValue::String(value) => println!("String: {}", value),
                    LiteralValue::Bool(value) => println!("Bool: {}", value),
                }
            },
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
            match &token.kind {
                TokenKind::Identifier(value) => {
                    match value.as_str() {
                        "print" => children.push(self.parse_function_call()?),
                        "let" => children.push(self.parse_variable_declaration()?),
                        _ => return Err(utils::get_error_message_with_location("PARSE002", token.row, token.col, &[])?),
                    }
                },
                TokenKind::EOF => {
                    self.tokens.next();
                    break;
                },
                _ => return Err(utils::get_error_message_with_location("PARSE002", token.row, token.col, &[])?),
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
        let token = self.tokens.next().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        let function_name = if let TokenKind::Identifier(name) = token.kind {
            name
        } else {
            return Err(get_error_message_with_location("PARSE004", token.row, token.col, &[])?);
        };

        match self.tokens.next() {
            Some(token) if token.kind == TokenKind::LParen => {},
            Some(token) => return Err(utils::get_error_message_with_location("PARSE005", token.row, token.col, &[])?),
            _ => return Err(utils::get_error_message("PARSE003", &[])?),
        };

        let argument = self.parse_argument()?;
        
        match self.tokens.next() {
            Some(token) if token.kind == TokenKind::RParen => {},
            Some(token) => return Err(utils::get_error_message_with_location("PARSE006", token.row, token.col, &[])?),
            _ => return Err(utils::get_error_message("PARSE003", &[])?),
        }

        match self.tokens.next(){
            Some(token) if token.kind == TokenKind::Semicolon => {},
            Some(token) => return Err(utils::get_error_message_with_location("PARSE007", token.row, token.col, &[])?),
            _ => return Err(utils::get_error_message("PARSE003", &[])?),
        }

        Ok(Node {
            kind: NodeKind::FunctionCall { name: function_name },
            children: vec![argument],
        })
    }

    /// 変数定義の構文解析
    fn parse_variable_declaration(&mut self) -> Result<Node, String> {
        self.tokens.next();
        let token = self.tokens.next().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        let name = if let TokenKind::Identifier(name) = token.kind {
            name
        } else {
            return Err(utils::get_error_message_with_location("PARSE013", token.row, token.col, &[])?);
        };
        
        let token = self.tokens.next().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        if token.kind != TokenKind::Equal {
            return Err(utils::get_error_message_with_location("PARSE014", token.row, token.col, &[])?);
        }

        let expression = self.parse_expression()?;

        match self.tokens.next() {
            Some(token) if token.kind == TokenKind::Semicolon => {},
            Some(token) => return Err(utils::get_error_message_with_location("PARSE015", token.row, token.col, &[])?),
            _ => return Err(utils::get_error_message("PARSE003", &[])?),
        }

        Ok(Node {
            kind: NodeKind::VariableDeclaration { name },
            children: vec![expression],
        })
    }

    /// 引数の構文解析
    fn parse_argument(&mut self) -> Result<Node, String> {
        let token = self.tokens.peek().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        match token.kind {
            TokenKind::StringLiteral(_) | TokenKind::NumberLiteral(_) | TokenKind::AddAndSubOperator(_) | TokenKind::LParen | TokenKind::BoolLiteral(_) | TokenKind::Identifier(_) => {
                let assignable = self.parse_assignable()?;
                Ok(Node {
                    kind: NodeKind::Argument,
                    children: vec![assignable],
                })
            },
            _ => Err(utils::get_error_message_with_location("PARSE008", token.row, token.col, &[])?),
        }
    }

    /// 割り当て可能値の構文解析（引数、代入式の右辺）
    fn parse_assignable(&mut self) -> Result<Node, String> {
        let token = self.tokens.peek().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        match token.kind.clone() {
            TokenKind::StringLiteral(_) | TokenKind::NumberLiteral(_) | TokenKind::BoolLiteral(_) 
            | TokenKind::AddAndSubOperator(_) | TokenKind::LParen | TokenKind::Identifier(_) => {
                return self.parse_expression();
            },
            _ => Err(utils::get_error_message_with_location("PARSE016", token.row, token.col, &[])?),
        }
    }

    /// 式の構文解析
    fn parse_expression(&mut self) -> Result<Node, String> {
        let token = self.tokens.peek().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        match token.kind {
            TokenKind::BoolLiteral(_) => return self.parse_literal(),
            _ => return self.parse_compare(),
        }
    }

    /// 比較式の構文解析
    fn parse_compare(&mut self) -> Result<Node, String> {
        let left = self.parse_value();
        let token = self.tokens.peek().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        let operator = if let TokenKind::CompareOperator(value) = token.kind.clone() {
            self.tokens.next();
            value
        } else {
            return left
        };

        let right = self.parse_value()?;
        return Ok(Node {
            kind: NodeKind::Compare { operator },
            children: vec![left?, right]
        });
    }

    /// 値の構文解析
    fn parse_value(&mut self) -> Result<Node, String> {
        let token = self.tokens.peek().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        match token.kind {
            TokenKind::NumberLiteral(_) | TokenKind::AddAndSubOperator(_) | TokenKind::LParen | TokenKind::Identifier(_) => {
                return self.parse_add_and_sub()
            },
            TokenKind::StringLiteral(_) => return self.parse_literal(),
            _ => { Err(utils::get_error_message_with_location("PARSE002", token.row, token.col, &[])?) },
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

    /// 掛け算、割り算の構文解析
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
        let token = self.tokens.peek().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        match token.kind.clone() {
            TokenKind::NumberLiteral(_) | TokenKind::LParen | TokenKind::Identifier(_) => {
                return self.parse_primary()
            },
            TokenKind::AddAndSubOperator(operator) => {
                self.tokens.next();
                let number = self.parse_primary()?;
                Ok(Node {
                    kind: NodeKind::Unary { operator: operator.to_string() },
                    children: vec![number],
                })
            },
            _ => { Err(utils::get_error_message_with_location("PARSE002", token.row, token.col, &[])?) },
        }
    }

    /// 数値、計算式の'()'の構文解析
    fn parse_primary(&mut self) -> Result<Node, String> {
        let token = self.tokens.peek().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        match token.kind{
            TokenKind::NumberLiteral(_) => return self.parse_literal(),
            TokenKind::LParen => {
                self.tokens.next();
                let expr = self.parse_add_and_sub();

                // 閉じカッコの確認
                let next_token = self.tokens.next().ok_or(utils::get_error_message("PARSE003", &[])?)?;
                match next_token.kind {
                    TokenKind::RParen => return expr,
                    _ => Err(utils::get_error_message_with_location("PARSE009", next_token.row, next_token.col, &[])?),
                }
            },
            TokenKind::Identifier(_) => return self.parse_variable(),
            _ => { Err(utils::get_error_message_with_location("PARSE002", token.row, token.col, &[])?) },
        }
    }

    /// 変数の構文解析
    fn parse_variable(&mut self) -> Result<Node, String> {
        let token = self.tokens.next().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        if let TokenKind::Identifier(name) = token.kind {
            Ok(Node {
                kind: NodeKind::Variable { name },
                children: vec![],
            })
        } else {
            Err(utils::get_error_message_with_location("PARSE017", token.row, token.col, &[])?)
        }
    }

    /// リテラル型の構文解析（String, Number, Bool）
    fn parse_literal(&mut self) -> Result<Node, String> {
        let token = self.tokens.next().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        match token.kind {
            TokenKind::StringLiteral(value) => {
                return Ok(Node {
                    kind: NodeKind::Literal(LiteralValue::String(value)), 
                    children: vec![]
                });
            },
            TokenKind::NumberLiteral(value) => {
                if let Ok(number) = value.parse::<f64>() {
                    return Ok(Node {
                        kind: NodeKind::Literal(LiteralValue::Float(number)),
                        children: vec![],
                    });
                }
            },
            TokenKind::BoolLiteral(value) => {
                match value.as_str() {
                    "true" => {
                        return Ok(Node {
                            kind: NodeKind::Literal(LiteralValue::Bool(true)),
                            children: vec![],
                        });
                    },
                    "false" => {
                        return Ok(Node {
                            kind: NodeKind::Literal(LiteralValue::Bool(false)),
                            children: vec![],
                        });
                    },
                    _ => {},
                }
            },
            _ => {}
        }
        
        Err(utils::get_error_message_with_location("PARSE010", token.row, token.col, &[])?)
    }
}

/// 構文解析を行う
pub fn parse(tokens: Vec<Token>) -> Result<Node, String> {
    let mut parser = Parser::new(tokens);
    let node = parser.parse_program()?;
    Ok(node)
}