use std::{iter::Peekable, vec::IntoIter};
use crate::{keyword::{BoolValue, Keyword}, lexical_analyzer::{Token, TokenKind}, operator::{Arithmetic, BinaryArithmetic, BinaryLogical, Comparison, Logical, UnaryArithmetic, UnaryLogical}, utils::{self, get_error_message_with_location}};

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralValue {
    Int(i32),
    Float(f64),
    String(String),
    Bool(bool),
}

#[derive(Debug, PartialEq, Clone)]
pub enum NodeKind {
    Program,
    FunctionCall { name: String },
    VariableDeclaration { name: String },
    VariableAssignment { name: String },
    Argument,
    Variable { name: String },
    Logical(Logical),
    Compare(Comparison),
    Arithmetic(Arithmetic),
    Literal(LiteralValue),
    IfStatement,
    FunctionDefinition { name: String, parameters: Vec<String> },
}
#[derive(Debug, Clone)]
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
            NodeKind::VariableAssignment { name } => println!("VariableAssignment: {}", name),
            NodeKind::Variable { name } => println!("Variable: {}", name),
            NodeKind::Logical(operator) => {
                match operator {
                    Logical::Binary(BinaryLogical::Or) => println!("Logical: or"),
                    Logical::Binary(BinaryLogical::And) => println!("Logial: and"),
                    Logical::Binary(BinaryLogical::Xor) => println!("Logical: xor"),
                    Logical::Unary(UnaryLogical::Not) => println!("Logical: not"),
                }
            },
            NodeKind::Compare(operator) => {
                let op_str = match operator {
                    Comparison::Equal => "==",
                    Comparison::NotEqual => "!=",
                    Comparison::Greater => ">",
                    Comparison::GreaterEqual => ">=",
                    Comparison::Less => "<",
                    Comparison::LessEqual => "<=",
                };
                println!("Compare: {}", op_str)
            },
            NodeKind::Arithmetic(operator) => {
                match operator {
                    Arithmetic::Binary(bin_op) => {
                        let op_chr = match bin_op {
                            BinaryArithmetic::Add => '+',
                            BinaryArithmetic::Subtract => '-',
                            BinaryArithmetic::Multiply => '*',
                            BinaryArithmetic::Divide => '/',
                        };
                        println!("BinaryArithmetic: {}", op_chr);
                    },
                    Arithmetic::Unary(_) => println!("UnaryArithmetic: -"),
                }
            },
            NodeKind::Literal ( literal ) => {
                match literal {
                    LiteralValue::Int(value) => println!("Int: {}", value),
                    LiteralValue::Float(value) => println!("Float: {}", value),
                    LiteralValue::String(value) => println!("String: {}", value),
                    LiteralValue::Bool(value) => println!("Bool: {}", value),
                }
            },
            NodeKind::IfStatement => {
                println!("IfStatement:");
                if let Some(condition) = self.children.get(0) {
                    for _ in 0..(depth + 1) {
                        print!("  ");
                    }
                    println!("Condition:");
                    condition.print(depth + 2); // 再帰的にツリーを表示
                }

                if let Some(then_block) = self.children.get(1) {
                    for _ in 0..(depth + 1) {
                        print!("  ");
                    }
                    println!("Then Block:");
                    then_block.print(depth + 2);
                }

                if let Some(else_block) = self.children.get(2) {
                    for _ in 0..(depth + 1) {
                        print!("  ");
                    }
                    println!("Else Block:");
                    else_block.print(depth + 2);
                }

                return;
            },
            NodeKind::FunctionDefinition { name, parameters } => {
                println!("FunctionDefinition: {}", name);
                for parameter in parameters {
                    for _ in 0..(depth + 1) {
                        print!("  ");
                    }
                    println!("parameter: {}", parameter);
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
    fn parse_program(&mut self, scope_end: Option<TokenKind>) -> Result<Node, String> {
        let mut children = Vec::new();

        while let Some(token) = self.tokens.peek() {
            if let Some(end) = &scope_end {
                if &token.kind == end {
                    break;
                }
            }
            match &token.kind {
                TokenKind::Identifier(_) => {
                    children.push(self.parse_identifier()?);
                },
                TokenKind::Keyword(keyword) => {
                    match keyword {
                        Keyword::Let => children.push(self.parse_variable_declaration()?),
                        Keyword::If => children.push(self.parse_if_statement()?),
                        Keyword::Function => children.push(self.parse_function_definition()?),
                        _ => {},
                    }
                }
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

    fn parse_if_statement(&mut self) -> Result<Node, String> {
        self.tokens.next();

        self.check_next_token(TokenKind::LParen)?;

        let condition = self.parse_expression()?;

        self.check_next_token(TokenKind::RParen)?;
        self.check_next_token(TokenKind::LBrace)?;

        let then_block = self.parse_program(Some(TokenKind::RBrace))?;

        self.check_next_token(TokenKind::RBrace)?;

        let mut children = vec![condition, then_block];

        match self.tokens.peek() {
            Some(token) if token.kind == TokenKind::Keyword(Keyword::Else) => {
                self.tokens.next();
                self.check_next_token(TokenKind::LBrace)?;
        
                let else_block = self.parse_program(Some(TokenKind::RBrace))?;
        
                self.check_next_token(TokenKind::RBrace)?;

                children.push(else_block);
            },
            _ => {},
        }

        Ok(Node {
            kind: NodeKind::IfStatement,
            children: children,
        })
    }

    fn parse_function_definition(&mut self) -> Result<Node, String> {
        self.tokens.next();

        let token = self.tokens.next().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        let function_name = match token.kind {
            TokenKind::Identifier(name) => name,
            _ => return Err(utils::get_error_message_with_location(
                "PARSE005", token.row, token.col, &[("token", "function_name")]
            )?),
        };
        self.check_next_token(TokenKind::LParen)?;

        let mut parameters = Vec::new();
        loop {
            let token = self.tokens.peek().ok_or(utils::get_error_message("PARSE003", &[])?)?;
            if token.kind == TokenKind::RParen { break; }

            let token = self.tokens.next().ok_or(utils::get_error_message("PARSE003", &[])?)?;
            let name = if let TokenKind::Identifier(name) = token.kind {
                name
            } else {
                return Err(utils::get_error_message_with_location("PARSE007", token.row, token.col, &[])?);
            };
            parameters.push(name);

            let token = self.tokens.peek().ok_or(utils::get_error_message("PARSE003", &[])?)?;
            match token.kind {
                TokenKind::Comma => { self.tokens.next(); },
                TokenKind::RParen => break,
                _ => return Err(utils::get_error_message_with_location("PARSE007", token.row, token.col, &[])?),
            }
        }

        self.check_next_token(TokenKind::RParen)?;
        self.check_next_token(TokenKind::LBrace)?;
        
        let block = self.parse_program(Some(TokenKind::RBrace))?;
        self.check_next_token(TokenKind::RBrace)?;

        Ok(Node {
            kind: NodeKind::FunctionDefinition{ name: function_name, parameters },
            children: vec![block],
        })

    }

    /// Identifierの構文解析
    /// 関数呼び出しと変数の再代入
    fn parse_identifier(&mut self) -> Result<Node, String> {
        let token = self.tokens.next().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        let identifier = if let TokenKind::Identifier(name) = token.kind {
            name
        } else {
            return Err(get_error_message_with_location("PARSE004", token.row, token.col, &[])?);
        };

        let token = self.tokens.next().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        match token.kind {
            TokenKind::LParen => {
                let arguments = self.parse_argument()?;
        
                self.check_next_token(TokenKind::RParen)?;
                self.check_next_token(TokenKind::Semicolon)?;
        
                Ok(Node {
                    kind: NodeKind::FunctionCall { name: identifier },
                    children: vec![arguments],
                })
            },
            TokenKind::Equal => {
                let expression = self.parse_expression()?;
        
                self.check_next_token(TokenKind::Semicolon)?;
        
                Ok(Node {
                    kind: NodeKind::VariableAssignment { name: identifier },
                    children: vec![expression],
                })
            },
            _ => Err(utils::get_error_message_with_location("PARSE002", token.row, token.col, &[])?),
        }
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
        
        self.check_next_token(TokenKind::Equal)?;

        let expression = self.parse_expression()?;

        self.check_next_token(TokenKind::Semicolon)?;

        Ok(Node {
            kind: NodeKind::VariableDeclaration { name },
            children: vec![expression],
        })
    }

    /// 引数の構文解析
    fn parse_argument(&mut self) -> Result<Node, String> {
        let mut arguments = Vec::new();
        loop {
            let token = self.tokens.peek().ok_or(utils::get_error_message("PARSE003", &[])?)?;
            if token.kind == TokenKind::RParen { break; }
            arguments.push(self.parse_assignable()?);

            let token = self.tokens.peek().ok_or(utils::get_error_message("PARSE003", &[])?)?;
            match token.kind {
                TokenKind::Comma => { self.tokens.next(); },
                TokenKind::RParen => break,
                _ => return Err(utils::get_error_message_with_location("PARSE007", token.row, token.col, &[])?),
            }
        }
        Ok(Node {
            kind: NodeKind::Argument,
            children: arguments,
        })
    }

    /// 割り当て可能値の構文解析（引数、代入式の右辺）
    fn parse_assignable(&mut self) -> Result<Node, String> {
        let token = self.tokens.peek().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        match token.kind.clone() {
            TokenKind::StringLiteral(_) | TokenKind::NumberLiteral(_) | TokenKind::BoolLiteral(_) 
            | TokenKind::ArithmeticOperator(Arithmetic::Unary(_)) | TokenKind::LParen | TokenKind::Identifier(_) => {
                return self.parse_expression();
            },
            _ => Err(utils::get_error_message_with_location("PARSE016", token.row, token.col, &[])?),
        }
    }

    /// 式の構文解析
    fn parse_expression(&mut self) -> Result<Node, String> {
        self.parse_logical()
    }

    /// 論理演算の構文解析
    fn parse_logical(&mut self) -> Result<Node, String> {
        self.parse_or_expr()
    }

    /// OR演算の構文解析
    fn parse_or_expr(&mut self) -> Result<Node, String> {
        let mut left = self.parse_and_expr()?;
        while let Some(TokenKind::LogicalOperator(Logical::Binary(BinaryLogical::Or))) = self.tokens.peek().map(|t| &t.kind) {
            self.tokens.next();
            let right = self.parse_and_expr()?;
            left = Node {
                kind: NodeKind::Logical(Logical::Binary(BinaryLogical::Or)),
                children: vec![left, right]
            };
        }
        Ok(left)
    }

    /// AND演算の構文解析
    fn parse_and_expr(&mut self) -> Result<Node, String> {
        let mut left = self.parse_not_expr()?;
        while let Some(TokenKind::LogicalOperator(operator)) = self.tokens.peek().map(|t| &t.kind) {
            let operator = operator.clone();
            if operator == Logical::Binary(BinaryLogical::And) || operator == Logical::Binary(BinaryLogical::Xor) {
                self.tokens.next();
                let right = self.parse_not_expr()?;
                left = Node {
                    kind: NodeKind::Logical(operator),
                    children: vec![left, right]
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    /// NOT演算の構文解析
    fn parse_not_expr(&mut self) -> Result<Node, String> {
        let token = self.tokens.peek().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        match token.kind {
            TokenKind::LogicalOperator(Logical::Unary(UnaryLogical::Not)) => {
                let value = self.parse_expression()?;
                Ok(Node {
                    kind: NodeKind::Logical(Logical::Unary(UnaryLogical::Not)),
                    children: vec![value],
                })
            },
            TokenKind::BoolLiteral(_) => return self.parse_literal(),
            _ => return self.parse_compare(),
        }
    }

    /// 比較式の構文解析
    fn parse_compare(&mut self) -> Result<Node, String> {
        let left = self.parse_value();
        let token = self.tokens.peek().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        let operator = if let TokenKind::CompareOperator(value) = token.kind.clone() {
            value
        } else {
            return left
        };
        
        self.tokens.next();
        let right = self.parse_value()?;
        return Ok(Node {
            kind: NodeKind::Compare(operator),
            children: vec![left?, right]
        });
    }

    /// 値の構文解析
    fn parse_value(&mut self) -> Result<Node, String> {
        let token = self.tokens.peek().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        match token.kind {
            TokenKind::NumberLiteral(_) | TokenKind::ArithmeticOperator(Arithmetic::Unary(_))
            | TokenKind::LParen | TokenKind::Identifier(_) => {
                return self.parse_add_and_sub()
            },
            TokenKind::StringLiteral(_) => return self.parse_literal(),
            _ => { Err(utils::get_error_message_with_location("PARSE002", token.row, token.col, &[])?) },
        }
    }

    /// 足し算、引き算の構文解析
    fn parse_add_and_sub(&mut self) -> Result<Node, String> {
        let mut left = self.parse_mul_and_div()?;
        while let Some(TokenKind::ArithmeticOperator(Arithmetic::Unary(operator))) = self.tokens.peek().map(|t| &t.kind) {
            let operator = match operator {
                UnaryArithmetic::Plus => BinaryArithmetic::Add,
                UnaryArithmetic::Minus => BinaryArithmetic::Subtract,
            };
            self.tokens.next();
            let right = self.parse_mul_and_div()?;
            left = Node {
                kind: NodeKind::Arithmetic(Arithmetic::Binary(operator)),
                children: vec![left, right]
            };
        }
        Ok(left)
    }

    /// 掛け算、割り算の構文解析
    fn parse_mul_and_div(&mut self) -> Result<Node, String> {
        let mut left = self.parse_unary()?;
        while let Some(TokenKind::ArithmeticOperator(Arithmetic::Binary(operator))) = self.tokens.peek().map(|t| &t.kind) {
            let operator = operator.clone();
            self.tokens.next();
            let right = self.parse_unary()?;
            left = Node {
                kind: NodeKind::Arithmetic(Arithmetic::Binary(operator)),
                children: vec![left, right]
            };
        }
        Ok(left)
    }

    /// 単項演算子の構文解析
    fn parse_unary(&mut self) -> Result<Node, String> {
        let token = self.tokens.peek().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        match token.kind.clone() {
            TokenKind::NumberLiteral(_) | TokenKind::LParen | TokenKind::Identifier(_)
            | TokenKind::ArithmeticOperator(Arithmetic::Unary(UnaryArithmetic::Plus))=> {
                return self.parse_primary()
            },
            TokenKind::ArithmeticOperator(Arithmetic::Unary(UnaryArithmetic::Minus)) => {
                self.tokens.next();
                let number = self.parse_primary()?;
                Ok(Node {
                    kind: NodeKind::Arithmetic(Arithmetic::Unary(UnaryArithmetic::Minus)),
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

    /// 変数呼び出しの構文解析
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
                match value {
                    BoolValue::True => {
                        return Ok(Node {
                            kind: NodeKind::Literal(LiteralValue::Bool(true)),
                            children: vec![],
                        });
                    },
                    BoolValue::False => {
                        return Ok(Node {
                            kind: NodeKind::Literal(LiteralValue::Bool(false)),
                            children: vec![],
                        });
                    },
                }
            },
            _ => {}
        }
        
        Err(utils::get_error_message_with_location("PARSE010", token.row, token.col, &[])?)
    }

    fn check_next_token(&mut self, token_kind: TokenKind) -> Result<Token, String> {
        match self.tokens.next() {
            Some(token) if token.kind == token_kind => Ok(token),
            Some(token) => {
                let token_str = match token_kind {
                    TokenKind::LBrace => "{",
                    TokenKind::RBrace => "}",
                    TokenKind::LParen => "(",
                    TokenKind::RParen => ")",
                    TokenKind::Semicolon => ";",
                    TokenKind::Equal => "=",
                    _ => "文字",
                };

                Err(utils::get_error_message_with_location(
                    "PARSE005",
                    token.row,
                    token.col,
                    &[("token", token_str)]
                )?)
            },
            _ => Err(utils::get_error_message("PARSE003", &[])?),
        }
    }
}

/// 構文解析を行う
pub fn parse(tokens: Vec<Token>) -> Result<Node, String> {
    let mut parser = Parser::new(tokens);
    let node = parser.parse_program(None)?;
    Ok(node)
}