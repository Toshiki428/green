use std::{iter::Peekable, vec::IntoIter};
use crate::{
    lexer::{
        keyword::{BoolKeyword, Keyword},
        token::{Token, TokenKind},
    },
    parser::node::Node,
    common::{
        operator::*,
        types::{BlockType, LiteralValue},
        error_code::ErrorCode,
    },
    utils::error_message::ErrorMessage
};

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
        self.parse_statements(BlockType::Global)
    }

    fn parse_statements(&mut self, block_type: BlockType) -> Result<Node, String> {
        let scope_end = match block_type {
            BlockType::Conditional | BlockType::Function | BlockType::Loop => Some(TokenKind::RBrace),
            BlockType::Global => None,
        };
        let mut children = Vec::new();
        while let Some(token) = self.tokens.peek() {
            if let Some(end) = &scope_end {
                if &token.kind == end {
                    break;
                }
            }
            match &token.kind {
                TokenKind::Identifier(name) => {
                    let name = name.clone();
                    self.tokens.next();
                    let token = self.next_token()?;
                    match token.kind {
                        TokenKind::LParen => {
                            let arguments = self.parse_argument()?;
                    
                            self.check_next_token(TokenKind::RParen)?;
                            self.check_next_token(TokenKind::Semicolon)?;
                            
                            children.push(Node::FunctionCall {
                                name,
                                arguments,
                            });
                        },
                        TokenKind::Equal => {
                            let expression = self.parse_expression()?;
                    
                            self.check_next_token(TokenKind::Semicolon)?;
                    
                            children.push(Node::VariableAssignment {
                                name,
                                expression: Box::new(expression),
                            })
                        },
                        _ => return Err(ErrorMessage::global().get_error_message_with_location(
                            &ErrorCode::Parse002,
                            token.row, token.col,
                            &[("token", &token.kind.to_string())],
                        )?),
                    }
                },
                TokenKind::Keyword(keyword) => {
                    match keyword {
                        Keyword::Let => {
                            self.tokens.next();
                            let name_token = self.next_token()?;
                            let name = match name_token.kind {
                                TokenKind::Identifier(name) => name,
                                _ => return Err(ErrorMessage::global().get_error_message_with_location(
                                    &ErrorCode::Parse005,
                                    name_token.row, name_token.col,
                                    &[("token", "変数名")],
                                )?),
                            };
                            self.check_next_token(TokenKind::Colon)?;
                            let type_token = self.next_token()?;
                            let variable_type = match type_token.kind {
                                TokenKind::VariableType(variable_type) => variable_type,
                                _ => return Err(ErrorMessage::global().get_error_message_with_location(
                                    &ErrorCode::Parse005,
                                    name_token.row, name_token.col,
                                    &[("token", "型")],
                                )?),
                            };
                            children.push(Node::VariableDeclaration {
                                name: name.to_string(),
                                variable_type,
                            });
                            let next_token = self.next_token()?;
                            if next_token.kind == TokenKind::Semicolon {
                            } else if next_token.kind == TokenKind::Equal {
                                let expression = self.parse_assignable()?;
                                self.check_next_token(TokenKind::Semicolon)?;
                                children.push(Node::VariableAssignment {
                                    name,
                                    expression: Box::new(expression),
                                });
                            } else {
                                return Err(ErrorMessage::global().get_error_message_with_location(
                                    &ErrorCode::Parse005,
                                    next_token.row, next_token.col,
                                    &[("token", ";")]
                                )?);
                            }
                        },
                        Keyword::If => children.push(self.parse_if_statement()?),
                        Keyword::While => children.push(self.parse_loop_statement()?),
                        Keyword::Function => children.push(self.parse_function_definition()?),
                        Keyword::Return => {
                            if block_type == BlockType::Function {
                                self.tokens.next();
                                let return_value = self.parse_assignable()?;
                                self.check_next_token(TokenKind::Semicolon)?;
                                children.push(Node::ReturnStatement { assignalbe: Box::new(return_value) });
                            }
                            else {
                                return Err(ErrorMessage::global().get_error_message_with_location(
                                    &ErrorCode::Parse002,
                                    token.row, token.col,
                                    &[("token", &keyword.to_string())],
                                )?);
                            }
                        },
                        _ => return Err(ErrorMessage::global().get_error_message_with_location(
                            &ErrorCode::Parse002,
                            token.row, token.col,
                            &[("token", &keyword.to_string())],
                        )?),
                    }
                }
                TokenKind::EOF => {
                    self.tokens.next();
                    break;
                },
                _ => return Err(ErrorMessage::global().get_error_message_with_location(
                    &ErrorCode::Parse002,
                    token.row, token.col,
                    &[("token", &token.kind.to_string())],
                )?),
            }
        }

        Ok(Node::Block {
            block_type,
            statements: children,
        })
    }

    fn parse_if_statement(&mut self) -> Result<Node, String> {
        self.tokens.next();

        self.check_next_token(TokenKind::LParen)?;

        let condition = self.parse_expression()?;

        self.check_next_token(TokenKind::RParen)?;
        self.check_next_token(TokenKind::LBrace)?;

        let then_block = self.parse_statements(BlockType::Conditional)?;

        self.check_next_token(TokenKind::RBrace)?;

        let else_block = match self.tokens.peek() {
            Some(token) if token.kind == TokenKind::Keyword(Keyword::Else) => {
                self.tokens.next();
                self.check_next_token(TokenKind::LBrace)?;
        
                let else_block = self.parse_statements(BlockType::Conditional)?;
        
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

    fn parse_loop_statement(&mut self) -> Result<Node, String> {
        self.tokens.next();
        self.check_next_token(TokenKind::LParen)?;

        let condition_node = self.parse_expression()?;

        self.check_next_token(TokenKind::RParen)?;
        self.check_next_token(TokenKind::LBrace)?;

        let block = self.parse_statements(BlockType::Loop)?;

        self.check_next_token(TokenKind::RBrace)?;
        Ok(Node::LoopStatement {
            condition_node: Box::new(condition_node),
            block: Box::new(block),
        })
    }

    fn parse_function_definition(&mut self) -> Result<Node, String> {
        self.tokens.next();
        let token = self.next_token()?;
        let function_name = match token.kind {
            TokenKind::Identifier(name) => name,
            _ => return Err(ErrorMessage::global().get_error_message_with_location(
                &ErrorCode::Parse005,
                token.row, token.col,
                &[("token", "関数名")]
            )?),
        };
        self.check_next_token(TokenKind::LParen)?;

        // 引数処理
        let mut parameters = Vec::new();
        loop {
            let token = self.peek_token()?;
            if token.kind == TokenKind::RParen { break; }

            let token = self.next_token()?;
            let name = match token.kind {
                TokenKind::Identifier(name) => name,
                _ => return Err(ErrorMessage::global().get_error_message_with_location(
                    &ErrorCode::Parse002,
                    token.row, token.col,
                    &[("token", &token.kind.to_string())],
                )?),
            };
            
            self.check_next_token(TokenKind::Colon)?;
            
            let type_token = self.next_token()?;
            let variable_type = match type_token.kind {
                TokenKind::VariableType(variable_type) => variable_type,
                _ => return Err(ErrorMessage::global().get_error_message_with_location(
                    &ErrorCode::Parse002,
                    type_token.row, type_token.col,
                    &[("token", &type_token.kind.to_string())],
                )?),
            };
            let param = Node::VariableDeclaration {
                name,
                variable_type,
            };
            parameters.push(param);
            
            let token = self.peek_token()?;
            match token.kind {
                TokenKind::Comma => { self.tokens.next(); },
                TokenKind::RParen => break,
                _ => return Err(ErrorMessage::global().get_error_message_with_location(
                    &ErrorCode::Parse002,
                    token.row, token.col,
                    &[("token", &token.kind.to_string())],
                )?),
            }
        }

        self.check_next_token(TokenKind::RParen)?;
        self.check_next_token(TokenKind::LBrace)?;
        
        let block = self.parse_statements(BlockType::Function)?;
        self.check_next_token(TokenKind::RBrace)?;

        Ok(Node::FunctionDefinition {
            name: function_name,
            parameters,
            block: Box::new(block),
        })

    }

    /// 引数の構文解析
    fn parse_argument(&mut self) -> Result<Vec<Node>, String> {
        let mut arguments = Vec::new();
        loop {
            let token = self.peek_token()?;
            if token.kind == TokenKind::RParen { break; }
            arguments.push(self.parse_assignable()?);

            let token = self.peek_token()?;
            match token.kind {
                TokenKind::Comma => { self.tokens.next(); },
                TokenKind::RParen => break,
                _ => return Err(ErrorMessage::global().get_error_message_with_location(
                    &ErrorCode::Parse002,
                    token.row, token.col,
                    &[("token", &token.kind.to_string())],
                )?),
            }
        }
        Ok(arguments)
    }

    /// 割り当て可能値の構文解析（引数、代入式の右辺）
    fn parse_assignable(&mut self) -> Result<Node, String> {
        let token = self.peek_token()?;
        match token.kind.clone() {
            TokenKind::StringLiteral(_) | TokenKind::NumberLiteral(_) | TokenKind::BoolLiteral(_) 
            | TokenKind::ArithmeticOperator(Arithmetic::Unary(_)) | TokenKind::LParen => {
                return self.parse_expression();
            },
            TokenKind::Identifier(name) => {
                let next_token = self.peek_n(1)?;
                match next_token.kind {
                    TokenKind::LParen => {
                        self.tokens.next();
                        self.tokens.next();
                        let arguments = self.parse_argument()?;
                        
                        self.check_next_token(TokenKind::RParen)?;
                        
                        return Ok(Node::FunctionCallWithReturn {
                            name,
                            arguments,
                        });
                    },
                    _ => {
                        return self.parse_expression();
                    }
                }
            },
            _ => Err(ErrorMessage::global().get_error_message_with_location(
                &ErrorCode::Parse002,
                token.row, token.col,
                &[("token", &token.kind.to_string())],
            )?),
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
            match operator {
                Logical::Binary(BinaryLogical::And) | Logical::Binary(BinaryLogical::Xor) => {
                    self.tokens.next();
                    let right = self.parse_not_expr()?;
                    left = Node::Logical {
                        operator,
                        left: Box::new(left),
                        right: Some(Box::new(right)),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// NOT演算の構文解析
    fn parse_not_expr(&mut self) -> Result<Node, String> {
        let token = self.peek_token()?;
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
        let token = self.peek_token()?;
        let operator = match token.kind {
            TokenKind::CompareOperator(value) => value,
            _ => return left,
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
        let token = self.peek_token()?;
        match token.kind {
            TokenKind::NumberLiteral(_) | TokenKind::ArithmeticOperator(Arithmetic::Unary(_))
            | TokenKind::LParen | TokenKind::Identifier(_) => {
                return self.parse_add_and_sub()
            },
            TokenKind::StringLiteral(_) => return self.parse_literal(),
            _ => Err(ErrorMessage::global().get_error_message_with_location(
                &ErrorCode::Parse002,
                token.row, token.col,
                &[("token", &token.kind.to_string())],
            )?),
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
        let token = self.peek_token()?;
        match token.kind {
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
            _ => Err(ErrorMessage::global().get_error_message_with_location(
                &ErrorCode::Parse002,
                token.row,
                token.col,
                &[("token", &token.kind.to_string())]
            )?),
        }
    }

    /// 数値、計算式の'()'の構文解析
    fn parse_primary(&mut self) -> Result<Node, String> {
        let token = self.peek_token()?;
        match token.kind{
            TokenKind::NumberLiteral(_) => return self.parse_literal(),
            TokenKind::LParen => {
                self.tokens.next();
                let expr = self.parse_add_and_sub();

                self.check_next_token(TokenKind::RParen)?;
                return expr;
            },
            TokenKind::Identifier(_) => return self.parse_variable(),
            _ => Err(ErrorMessage::global().get_error_message_with_location(
                &ErrorCode::Parse002,
                token.row,
                token.col,
                &[("token", &token.kind.to_string())]
            )?),
        }
    }

    /// 変数呼び出しの構文解析
    fn parse_variable(&mut self) -> Result<Node, String> {
        let token = self.next_token()?;
        match token.kind {
            TokenKind::Identifier(name) => Ok(Node::Variable { name }),
            _ => Err(ErrorMessage::global().get_error_message_with_location(
                &ErrorCode::Parse002,
                token.row,
                token.col,
                &[("token", &token.kind.to_string())]
            )?),
        }
    }

    /// リテラル型の構文解析（String, Number, Bool）
    fn parse_literal(&mut self) -> Result<Node, String> {
        let token = self.next_token()?;
        match token.kind {
            TokenKind::StringLiteral(value) => {
                return Ok(Node::Literal { value: LiteralValue::String(value) });
            },
            TokenKind::NumberLiteral(integer) => {
                if let Ok(int_value) = integer.parse::<i32>() {
                    let token = self.peek_token()?;
                    match token.kind {
                        TokenKind::Dot => {
                            self.tokens.next();
                            let mut number_string = integer.clone() + ".";
                            let token = self.next_token()?;
                            if let TokenKind::NumberLiteral(decimal) = token.kind {
                                number_string.push_str(&decimal);
                                if let Ok(float_value) = number_string.parse::<f64>() {
                                    return Ok(Node::Literal { value: LiteralValue::Float(float_value) });
                                }
                            }
                            return Err(ErrorMessage::global().get_error_message_with_location(
                                &ErrorCode::Parse004,
                                token.row, token.col,
                                &[("number", &number_string)],
                            )?)
                        },
                        _ => return Ok(Node::Literal { value: LiteralValue::Int(int_value) })
                    }
                } else {
                    return Err(ErrorMessage::global().get_error_message_with_location(
                        &ErrorCode::Parse004,
                        token.row, token.col,
                        &[("number", &integer)],
                    )?)
                }
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
            _ => return Err(ErrorMessage::global().get_error_message_with_location(
                &ErrorCode::Parse002,
                token.row,
                token.col,
                &[("token", &token.kind.to_string())]
            )?),
        }
    }

    fn peek_token(&mut self) -> Result<Token, String> {
        match self.tokens.peek() {
            Some(token) => Ok(token.clone()),
            None => Err(ErrorMessage::global().get_error_message(&ErrorCode::Parse003, &[])?),
        }
    }

    fn next_token(&mut self) -> Result<Token, String> {
        match self.tokens.next() {
            Some(token) => Ok(token),
            None => Err(ErrorMessage::global().get_error_message(&ErrorCode::Parse003, &[])?),
        }
    }

    fn check_next_token(&mut self, token_kind: TokenKind) -> Result<Token, String> {
        let token = self.next_token()?;
        if token.kind == token_kind {
            Ok(token)
        } else {
            let kind_str = &token.kind.to_string();

            Err(ErrorMessage::global().get_error_message_with_location(
                &ErrorCode::Parse005,
                token.row,
                token.col,
                &[("token", kind_str)]
            )?)
        }
    }

    /// n+1個先のTokenを確認
    /// 
    /// n=0のとき1個先、n=1のとき2個先
    fn peek_n(&mut self, n: usize) -> Result<Token, String> {
        match self.tokens.by_ref().clone().nth(n) {
            Some(token) => Ok(token),
            None => Err(ErrorMessage::global().get_error_message(&ErrorCode::Parse003, &[])?)
        }
    }
}

/// 構文解析を行う
pub fn parse(tokens: Vec<Token>) -> Result<Node, String> {
    let mut parser = Parser::new(tokens);
    let node = parser.parse_program()?;
    Ok(node)
}