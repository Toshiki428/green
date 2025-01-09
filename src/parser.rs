use std::{iter::Peekable, vec::IntoIter};
use crate::{keyword::{BoolKeyword, Keyword}, lexical_analyzer::{Token, TokenKind}, operator::{Arithmetic, BinaryArithmetic, BinaryLogical, Comparison, Logical, UnaryArithmetic, UnaryLogical}, types::{LiteralValue, Type}, utils::{self, get_error_message_with_location}};

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    /// プログラム全体のノード
    Program {
        statements: Vec<Node>,
    },
    /// 関数呼び出し
    FunctionCall { 
        name: String,
        arguments: Vec<Node>,
    },
    /// 変数宣言
    VariableDeclaration {
        name: String,
        variable_type: Type,
    },
    /// 変数代入
    VariableAssignment {
        name: String,
        expression: Box<Node>,
    },
    /// 変数呼び出し
    Variable {
        name: String,
    },
    /// 論理演算
    Logical {
        operator: Logical,
        left: Box<Node>,
        right: Option<Box<Node>>,
    },
    /// 比較演算
    Compare {
        operator: Comparison,
        left: Box<Node>,
        right: Box<Node>,
    },
    /// 算術演算
    Arithmetic {
        operator: Arithmetic,
        left: Box<Node>,
        right: Option<Box<Node>>,
    },
    /// リテラル値
    Literal {
        value: LiteralValue,
    },
    /// If文
    IfStatement {
        condition_node: Box<Node>,
        then_block: Box<Node>,
        else_block: Option<Box<Node>>,
    },
    /// 関数定義
    FunctionDefinition {
        name: String,
        parameters: Vec<Node>,
        block: Box<Node>,
    },
}

impl Node {
    /// デバッグ用のprint文
    pub fn print(&self, depth: i32) {
        self.indent(depth);
        match self {
            Self::Program { statements } => {
                println!("program:");
                for statement in statements {
                    statement.print(depth+1);
                }
            },
            Self::FunctionCall { name, arguments } => {
                println!("FunctionCall: {}", name);
                self.indent(depth+1);
                println!("Args:");
                for argument in arguments {
                    argument.print(depth+2);
                }
            },
            Self::VariableDeclaration { name, variable_type } => {
                println!("VariableDeaclaration: {} ({})", name, variable_type.to_string());
            },
            Self::VariableAssignment { name, expression } => {
                println!("VariableAssignment: {}", name);
                expression.print(depth+1);
            },
            Self::Variable { name } => {
                println!("Variable: {}", name);
            },
            Self::Logical { operator, left, right } => {
                println!("operator: {}", operator.as_str());
                self.indent(depth+1);
                println!("left:");
                left.print(depth+2);
                if let Some(node) = right {
                    self.indent(depth+1);
                    println!("right:");
                    node.print(depth+2);
                }
            },
            Self::Compare { operator, left, right } => {
                println!("operator: {}", operator.as_str());
                self.indent(depth+1);
                println!("left:");
                left.print(depth+2);
                self.indent(depth+1);
                println!("right:");
                right.print(depth+2);
            },
            Self::Arithmetic { operator, left, right } => {
                println!("operator: {}", operator.as_str());
                self.indent(depth+1);
                println!("left:");
                left.print(depth+2);
                if let Some(node) = right {
                    self.indent(depth+1);
                    println!("right:");
                    node.print(depth+2);
                }
            },
            Self::Literal { value } => println!("Literal: {}", value.to_string()),
            Self::IfStatement { condition_node, then_block, else_block } => {
                println!("IfStatement:");
                self.indent(depth+1);
                println!("condition_node");
                condition_node.print(depth+2);
                self.indent(depth+1);
                println!("then_block");
                then_block.print(depth+2);
                if let Some(node) = else_block {
                    self.indent(depth+1);
                    println!("else_block");
                    node.print(depth+2);
                }
            },
            Self::FunctionDefinition { name, parameters, block } => {
                println!("FunctionDefinition: {}", name);
                for param in parameters {
                    param.print(depth+1);
                }
                self.indent(depth+1);
                println!("block:");
                block.print(depth+2);
            },
        }
    }

    fn indent(&self, depth: i32) {
        for _ in 0..depth {
            print!("  ");
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
                        Keyword::Let => {
                            self.tokens.next();
                            let name_token = self.tokens.next().ok_or(utils::get_error_message("PARSE003", &[])?)?;
                            let name = if let TokenKind::Identifier(name) = name_token.kind {
                                name
                            } else {
                                return Err(format!("不正な変数名{}, {}", name_token.row, name_token.col));
                            };
                            self.check_next_token(TokenKind::Colon)?;
                            let type_token = self.tokens.next().ok_or(utils::get_error_message("PARSE003", &[])?)?;
                            let variable_type = match type_token.kind {
                                TokenKind::VariableType(variable_type) => variable_type,
                                _ => return Err(format!("不正な型: {:?}", type_token.kind)),
                            };
                            children.push(Node::VariableDeclaration {
                                name: name.to_string(),
                                variable_type,
                            });
                            let next_token = self.tokens.next().ok_or(utils::get_error_message("PARSE003", &[])?)?;
                            if next_token.kind == TokenKind::Semicolon {
                            } else if next_token.kind == TokenKind::Equal {
                                let expression = self.parse_expression()?;
                                self.check_next_token(TokenKind::Semicolon)?;
                                children.push(Node::VariableAssignment {
                                    name,
                                    expression: Box::new(expression),
                                });
                            } else {
                                return Err("定義されていない関数宣言".to_string());
                            }

                            
                        },
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

        Ok(Node::Program {
            statements: children
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

        let else_block = match self.tokens.peek() {
            Some(token) if token.kind == TokenKind::Keyword(Keyword::Else) => {
                self.tokens.next();
                self.check_next_token(TokenKind::LBrace)?;
        
                let else_block = self.parse_program(Some(TokenKind::RBrace))?;
        
                self.check_next_token(TokenKind::RBrace)?;

                Some(else_block)
            },
            _ => None,
        };

        Ok(Node::IfStatement {
            condition_node: Box::new(condition),
            then_block: Box::new(then_block),
            else_block: else_block.map(Box::new),
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
            
            self.check_next_token(TokenKind::Colon)?;
            
            let type_token = self.tokens.next().ok_or(utils::get_error_message("PARSE003", &[])?)?;
            let variable_type = match type_token.kind {
                TokenKind::VariableType(variable_type) => variable_type,
                _ => return Err(format!("不正な型: {:?}", type_token.kind)),
            };
            let param = Node::VariableDeclaration {
                name,
                variable_type,
            };
            parameters.push(param);
            
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

        Ok(Node::FunctionDefinition {
            name: function_name,
            parameters,
            block: Box::new(block),
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
        
                Ok(Node::FunctionCall {
                    name: identifier,
                    arguments: arguments,
                })
            },
            TokenKind::Equal => {
                let expression = self.parse_expression()?;
        
                self.check_next_token(TokenKind::Semicolon)?;
        
                Ok(Node::VariableAssignment {
                    name: identifier,
                    expression: Box::new(expression),
                })
            },
            _ => Err(utils::get_error_message_with_location("PARSE002", token.row, token.col, &[])?),
        }
    }

    /// 引数の構文解析
    fn parse_argument(&mut self) -> Result<Vec<Node>, String> {
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
        Ok(arguments)
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
            left = Node::Logical {
                operator: Logical::Binary(BinaryLogical::Or),
                left: Box::new(left),
                right: Some(Box::new(right)),
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
                left = Node::Logical {
                    operator,
                    left: Box::new(left),
                    right: Some(Box::new(right)),
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
                Ok(Node::Logical {
                    operator: Logical::Unary(UnaryLogical::Not),
                    left: Box::new(value),
                    right: None,
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
        return Ok(Node::Compare {
            operator,
            left: Box::new(left?),
            right: Box::new(right),
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
            left = Node::Arithmetic {
                operator: Arithmetic::Binary(operator),
                left: Box::new(left),
                right: Some(Box::new(right)),
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
            left = Node::Arithmetic {
                operator: Arithmetic::Binary(operator),
                left: Box::new(left),
                right: Some(Box::new(right)),
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
                Ok(Node::Arithmetic {
                    operator: Arithmetic::Unary(UnaryArithmetic::Minus),
                    left: Box::new(number),
                    right: None,
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
            Ok(Node::Variable { name })
        } else {
            Err(utils::get_error_message_with_location("PARSE017", token.row, token.col, &[])?)
        }
    }

    /// リテラル型の構文解析（String, Number, Bool）
    fn parse_literal(&mut self) -> Result<Node, String> {
        let token = self.tokens.next().ok_or(utils::get_error_message("PARSE003", &[])?)?;
        match token.kind {
            TokenKind::StringLiteral(value) => {
                return Ok(Node::Literal { value: LiteralValue::String(value) });
            },
            TokenKind::NumberLiteral(integer) => {
                if let Ok(int_value) = integer.parse::<i32>() {
                    if TokenKind::Dot == self.tokens.peek().ok_or(utils::get_error_message("PARSE003", &[])?)?.kind {
                        self.tokens.next();
                        let mut number_string = integer.clone() + ".";
                        if let TokenKind::NumberLiteral(decimal) = self.tokens.next().ok_or(utils::get_error_message("PARSE003", &[])?)?.kind {
                            number_string.push_str(&decimal);
                            if let Ok(float_value) = number_string.parse::<f64>() {
                                return Ok(Node::Literal { value: LiteralValue::Float(float_value) });
                            }
                        } else {
                            return Err(format!("不正な数値: {}", number_string))
                        }
                    } else {
                        return Ok(Node::Literal { value: LiteralValue::Int(int_value) })
                    }
                }
                return Err(format!("不正な数値: {}", &integer))
            },
            TokenKind::BoolLiteral(value) => {
                match value {
                    BoolKeyword::True => {
                        return Ok(Node::Literal { value: LiteralValue::Bool(true) });
                    },
                    BoolKeyword::False => {
                        return Ok(Node::Literal { value: LiteralValue::Bool(false) });
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
                    TokenKind::Colon => ":",
                    TokenKind::Comma => ",",
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