use std::{iter::Peekable, vec::IntoIter};
use regex::Regex;
use super::node::*;
use crate::{
    common::{
        keyword::*,
        operator::*,
        types::{BlockType, LiteralValue, Type},
    }, error::{
        error_code::ErrorCode,
        error_context::ErrorContext,
    }, lexer::token::{Token, TokenKind}
};

struct Parser {
    tokens: Peekable<IntoIter<Token>>,
    block_stack: Vec<BlockType>,
    errors: Vec<ErrorContext>,
    doc_comment: String,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self{
            tokens: tokens.into_iter().peekable(),
            block_stack: Vec::new(),
            errors: Vec::new(),
            doc_comment: String::new(),
        }
    }

    /// ルートの構文解析
    fn parse_program(&mut self) -> RootNode {
        self.push_block(BlockType::Global);
        let program = self.parse_global();
        self.pop_block();
        program
    }

    fn parse_global(&mut self) -> RootNode {
        let mut functions = Vec::new();
        let mut coroutines = Vec::new();

        loop {
            let token = match self.peek_token(){
                Ok(token) => token,
                Err(e) => {
                    self.errors.push(e);
                    break;
                },
            };
            if matches!(&token.kind, TokenKind::EOF) {
                break;
            }

            if matches!(&token.kind, TokenKind::DocComment(_)) {
                match self.parse_doc_comment(&token) {
                    Ok(result) => {
                        match result {
                            Some(_) => {
                                self.errors.push(
                                    ErrorContext::new(
                                        ErrorCode::Parse006,
                                        Some(token.row), Some(token.col),
                                        vec![("statement", "プロセスコメント"), ("block", "関数")],
                                    )
                                );
                            },
                            None => {},
                        }
                        continue;
                    },
                    Err(e) => {
                        self.errors.push(e);
                        break;
                    }
                }
            }

            match token.kind {
                TokenKind::DeclarationKeyword(keyword) => {
                    match keyword {
                        DeclarationKeyword::Function => {
                            let function = match self.parse_function_definition() {
                                Ok(node) => node,
                                Err(e) => {
                                    self.errors.push(e);
                                    break;
                                },
                            };
                            functions.push(function);
                        },
                        DeclarationKeyword::Coroutine => {
                            let coroutine = match self.parse_coroutine_definition() {
                                Ok(node) => node,
                                Err(e) => {
                                    self.errors.push(e);
                                    break;
                                }
                            };
                            coroutines.push(coroutine);
                        },
                        _ => {
                            self.errors.push(
                                ErrorContext::new(
                                    ErrorCode::Parse006,
                                    Some(token.row), Some(token.col),
                                    vec![("statement", &keyword.to_string()), ("block", "関数")],
                                )
                            );
                            break;
                        },
                    }
                },
                _ => {
                    self.errors.push(
                        ErrorContext::new(
                            ErrorCode::Parse002,
                            Some(token.row), Some(token.col),
                            vec![("token", &token.kind.to_string())],
                        )
                    );
                    break;
                },
            }
        
            self.doc_comment = String::new();
        }

        RootNode { functions, coroutines }
    }

    fn parse_function_definition(&mut self) -> Result<FunctionDefinitionNode, ErrorContext> {
        let token = self.next_token()?;
        if self.block_stack.last() != Some(&BlockType::Global) {
            self.errors.push(ErrorContext::new(
                ErrorCode::Parse006,
                Some(token.row), Some(token.col),
                vec![
                    ("statement", "function"),
                    ("block", "global_block"),
                ],
            ))
        }

        let doc = self.get_doc_comment();
        
        let token = self.next_token()?;
        let function_name = match token.kind {
            TokenKind::Identifier(name) => name,
            _ => {
                self.errors.push(ErrorContext::new(
                    ErrorCode::Parse005,
                    Some(token.row), Some(token.col),
                    vec![("token", "関数名")]
                ));
                "None".to_string()
            },
        };
        self.check_next_token(TokenKind::LParen);

        // 引数処理
        let mut parameters = Vec::new();
        loop {
            let token = self.peek_token()?;
            if token.kind == TokenKind::RParen { break; }

            let token = self.next_token()?;
            let name = match token.kind {
                TokenKind::Identifier(name) => name,
                _ => {
                    self.errors.push(ErrorContext::new(
                        ErrorCode::Parse002,
                        Some(token.row), Some(token.col),
                        vec![("token", &token.kind.to_string())],
                    ));
                    "None".to_string()
                },
            };
            
            self.check_next_token(TokenKind::Colon);
            
            let type_token = self.next_token()?;
            let variable_type = match type_token.kind {
                TokenKind::TypeName(type_name) => Type::from_keyword(&type_name),
                _ => {
                    self.errors.push(ErrorContext::new(
                        ErrorCode::Parse002,
                        Some(type_token.row), Some(type_token.col),
                        vec![("token", &type_token.kind.to_string())],
                    ));
                    Type::Bool
                },
            };
            
            let token = self.peek_token()?;
            match token.kind {
                TokenKind::Comma => { self.next_token()?; },
                TokenKind::RParen => {
                    let param = ParameterNode {
                        name,
                        variable_type,
                    };
                    parameters.push(param);
                    break;
                },
                _ => {
                    self.errors.push(ErrorContext::new(
                        ErrorCode::Parse002,
                        Some(token.row), Some(token.col),
                        vec![("token", &token.kind.to_string())],
                    ));
                    self.next_token()?;
                },
            }

            let param = ParameterNode {
                name,
                variable_type,
            };
            parameters.push(param);
        }

        self.check_next_token(TokenKind::RParen);

        let next_token = self.next_token()?;
        let return_type = match next_token.kind {
            TokenKind::LBrace => {
                None
            },
            TokenKind::RArrow => {
                let type_token = self.next_token()?;
                match type_token.kind {
                    TokenKind::TypeName(type_name) => {
                        self.check_next_token(TokenKind::LBrace);
                        Some(Type::from_keyword(&type_name))
                    },
                    _ => {
                        return Err(ErrorContext::new(
                            ErrorCode::Parse002,
                            Some(type_token.row), Some(type_token.col),
                            vec![("token", &type_token.kind.to_string())],
                        ))
                    },
                }
            },
            _ => {
                return Err(ErrorContext::new(
                    ErrorCode::Parse002,
                    Some(next_token.row), Some(next_token.col),
                    vec![("token", &next_token.kind.to_string())],
                ))
            },
        };
        
        self.push_block(BlockType::Function);
        let block = self.parse_statements(BlockType::Function);
        self.pop_block();

        self.check_next_token(TokenKind::RBrace);

        Ok(FunctionDefinitionNode {
            name: function_name,
            parameters,
            return_type,
            block,
            doc,
        })

    }

    fn parse_coroutine_definition(&mut self) -> Result<CoroutineDefinitionNode, ErrorContext> {
        let token = self.next_token()?;
        if self.block_stack.last() != Some(&BlockType::Global) {
            self.errors.push(ErrorContext::new(
                ErrorCode::Parse006,
                Some(token.row), Some(token.col),
                vec![
                    ("statement", "coroutine"),
                    ("block", "global_block"),
                ],
            ))
        }

        let doc = self.get_doc_comment();
        
        let token = self.next_token()?;
        let coroutine_name = match token.kind {
            TokenKind::Identifier(name) => name,
            _ => {
                self.errors.push(ErrorContext::new(
                    ErrorCode::Parse005,
                    Some(token.row), Some(token.col),
                    vec![("token", "コルーチン名")]
                ));
                "None".to_string()
            },
        };
        self.check_next_token(TokenKind::LParen);

        self.check_next_token(TokenKind::RParen);
        self.check_next_token(TokenKind::LBrace);
        
        self.push_block(BlockType::Coroutine);
        let block = self.parse_statements(BlockType::Coroutine);
        self.pop_block();

        self.check_next_token(TokenKind::RBrace);
        Ok(CoroutineDefinitionNode { name: coroutine_name, block, doc })
    }

    fn parse_statements(&mut self, block_type: BlockType) -> BlockNode {
        let scope_end = match block_type {
            BlockType::Conditional | BlockType::Coroutine | BlockType::Function | BlockType::Loop => Some(TokenKind::RBrace),
            BlockType::Global => None,
        };
        let mut statements = Vec::new();
        loop {
            let token = match self.peek_token(){
                Ok(token) => token,
                Err(node) => {
                    self.errors.push(node);
                    break;
                },
            };
            if let Some(end) = &scope_end {
                if &token.kind == end {
                    break;
                }
            }
            if matches!(&token.kind, TokenKind::EOF) {
                break;
            }

            let token = token.clone();
            if matches!(&token.kind, TokenKind::DocComment(_)) {
                match self.parse_doc_comment(&token) {
                    Ok(result) => {
                        match result {
                            Some(node) => statements.push(node),
                            None => {},
                        }
                        continue;
                    },
                    Err(e) => {
                        self.errors.push(e);
                        break;
                    }
                }
            }

            let statement = self.parse_statement(token);
            match statement {
                Ok(node) => statements.push(node),
                Err(e) => {
                    self.errors.push(e);
                    break;
                },
            }
            self.doc_comment = String::new();
        }

        return BlockNode {
            block_type,
            statements,
        }
    }

    fn parse_statement(&mut self, token: Token) -> Result<PrivateNode, ErrorContext> {
        match token.kind {
            TokenKind::Identifier(name) => self.parse_identifier(name),
            TokenKind::ControlKeyword(keyword) => {
                match keyword {
                    ControlKeyword::If => self.parse_if_statement(),
                    ControlKeyword::While => self.parse_loop_statement(),

                    _ => return Err(ErrorContext::new(
                        ErrorCode::Parse002,
                        Some(token.row), Some(token.col),
                        vec![("token", &keyword.to_string())],
                    )),
                }
            },
            TokenKind::DeclarationKeyword(keyword) => self.parse_declaration_keyword(keyword, token.row, token.col),
            TokenKind::FunctionControl(keyword) => {
                if !self.block_stack.contains(&BlockType::Function) {
                    self.errors.push(ErrorContext::new(
                        ErrorCode::Parse006,
                        Some(token.row), Some(token.col),
                        vec![
                            ("statement", &keyword.to_string()),
                            ("block", "function"),
                        ],
                    ))
                }
                match keyword {
                    FunctionControl::Return => {
                        self.next_token()?;
                        let return_value = match self.parse_assignable() {
                            Ok(node) => node,
                            Err(e) => {
                                self.errors.push(e);
                                PrivateNode::Error
                            },
                        };
                        self.check_next_token(TokenKind::Semicolon);
                        Ok(PrivateNode::ReturnStatement {
                            assignalbe: Box::new(return_value),
                        })
                    },
                }
            },
            TokenKind::LoopControl(keyword) => {
                if !self.block_stack.contains(&BlockType::Loop) {
                    self.errors.push(ErrorContext::new(
                        ErrorCode::Parse006,
                        Some(token.row), Some(token.col),
                        vec![
                            ("statement", &keyword.to_string()),
                            ("block", "loop"),
                        ],
                    ))
                }
                
                self.next_token()?;
                self.check_next_token(TokenKind::Semicolon);

                let node = match keyword {
                    LoopControl::Break => PrivateNode::Break,
                    LoopControl::Continue => PrivateNode::Continue,
                };

                Ok(node)
            },
            TokenKind::CoroutineControl(keyword) => {
                self.next_token()?;

                match keyword {
                    CoroutineControl::Resume => {
                        let name_token = self.next_token()?;
                        let task_name = match name_token.kind {
                            TokenKind::Identifier(name) => name,
                            _ => return Err(ErrorContext::new(
                                ErrorCode::Parse005,
                                Some(name_token.row), Some(name_token.col),
                                vec![],
                            )),
                        };
                        self.check_next_token(TokenKind::Semicolon);
                        Ok(PrivateNode::CoroutineResume { task_name })
                    },
                    CoroutineControl::Yield => {
                        if !self.block_stack.contains(&BlockType::Coroutine) {
                            self.errors.push(ErrorContext::new(
                                ErrorCode::Parse006,
                                Some(token.row), Some(token.col),
                                vec![
                                    ("statement", &keyword.to_string()),
                                    ("block", "コルーチン"),
                                ],
                            ))
                        }
                        self.check_next_token(TokenKind::Semicolon);
                        Ok(PrivateNode::Yield)
                    },
                }
            },

            _ => return Err(ErrorContext::new(
                ErrorCode::Parse002,
                Some(token.row), Some(token.col),
                vec![("token", &token.kind.to_string())],
            )),
        }
    }

    fn parse_if_statement(&mut self) -> Result<PrivateNode, ErrorContext> {
        self.next_token()?;

        self.check_next_token(TokenKind::LParen);

        let condition = match self.parse_expression() {
            Ok(node) => node,
            Err(e) => {
                self.errors.push(e);
                PrivateNode::Error
            },
        };

        self.check_next_token(TokenKind::RParen);
        self.check_next_token(TokenKind::LBrace);

        self.push_block(BlockType::Conditional);
        let then_block = self.parse_statements(BlockType::Conditional);
        self.pop_block();

        self.check_next_token(TokenKind::RBrace);

        let else_block = match self.tokens.peek() {
            Some(token) if token.kind == TokenKind::ControlKeyword(ControlKeyword::Else) => {
                self.next_token()?;
                self.check_next_token(TokenKind::LBrace);
        
                self.push_block(BlockType::Conditional);
                let else_block = self.parse_statements(BlockType::Conditional);
                self.pop_block();
        
                self.check_next_token(TokenKind::RBrace);

                Some(else_block)
            },
            _ => None,
        };

        Ok(PrivateNode::IfStatement {
            condition_node: Box::new(condition),
            then_block,
            else_block,
        })
    }

    fn parse_loop_statement(&mut self) -> Result<PrivateNode, ErrorContext> {
        self.next_token()?;
        self.check_next_token(TokenKind::LParen);

        let condition_node = match self.parse_expression() {
            Ok(node) => node,
            Err(e) => {
                self.errors.push(e);
                PrivateNode::Error
            },
        };

        self.check_next_token(TokenKind::RParen);

        self.check_next_token(TokenKind::LBrace);

        self.push_block(BlockType::Loop);
        let block = self.parse_statements(BlockType::Loop);
        self.pop_block();

        self.check_next_token(TokenKind::RBrace);

        Ok(PrivateNode::LoopStatement {
            condition_node: Box::new(condition_node),
            block: block,
        })
    }

    fn parse_identifier(&mut self, name: String) -> Result<PrivateNode, ErrorContext> {
        self.next_token()?;  // 変数名または関数名のトークンをスキップスキップ
        let token = self.next_token()?;

        match token.kind {
            TokenKind::LParen => {  // 関数と判定
                let arguments = self.parse_argument();

                self.check_next_token(TokenKind::RParen);
                self.check_next_token(TokenKind::Semicolon);
                
                return Ok(PrivateNode::FunctionCall {
                    name,
                    arguments,
                    return_flg: false,
                });
            },
            TokenKind::Equal => {  // 変数と判定
                let expression = self.parse_expression()?;
                
                self.check_next_token(TokenKind::Semicolon);
        
                return Ok(PrivateNode::VariableAssignment {
                    name,
                    expression: Box::new(expression),
                })
            },
            _ => {
                return Err(ErrorContext::new(
                    ErrorCode::Parse002,
                    Some(token.row), Some(token.col),
                    vec![("token", &token.kind.to_string())],
                ))
            },
        }
    }

    fn parse_declaration_keyword(&mut self, keyword: DeclarationKeyword, row: u32, col: u32) -> Result<PrivateNode, ErrorContext> {
        match keyword {
            DeclarationKeyword::Let => {
                self.next_token()?;
                let name_token = self.next_token()?;
                let name = match name_token.kind {
                    TokenKind::Identifier(name) => name,
                    _ => {
                        self.errors.push(ErrorContext::new(
                            ErrorCode::Parse005,
                            Some(name_token.row), Some(name_token.col),
                            vec![("token", "変数名")],
                        ));
                        "None".to_string()
                    }
                };

                self.check_next_token(TokenKind::Colon);

                let type_token = self.next_token()?;
                let variable_type = match type_token.kind {
                    TokenKind::TypeName(type_name) => Type::from_keyword(&type_name),
                    _ => {
                        self.errors.push(ErrorContext::new(
                            ErrorCode::Parse005,
                            Some(name_token.row), Some(name_token.col),
                            vec![("token", "型")],
                        ));
                        Type::Bool
                    },
                };

                let next_token = self.next_token()?;
                let initializer = match next_token.kind {
                    TokenKind::Semicolon => None,
                    TokenKind::Equal => {
                        let expression = self.parse_assignable()?;
                        self.check_next_token(TokenKind::Semicolon);
                        Some(Box::new(expression))
                    },
                    _ =>{
                        self.errors.push(ErrorContext::new(
                            ErrorCode::Parse005,
                            Some(next_token.row), Some(next_token.col),
                            vec![("token", ";")],
                        ));
                        None
                    }
                };

                return Ok(PrivateNode::VariableDeclaration {
                    name: name.to_string(),
                    variable_type,
                    initializer,
                    doc: self.get_doc_comment(),
                });
            },
            DeclarationKeyword::Coro => {
                self.next_token()?;
                let name_token = self.next_token()?;
                let task_name = match name_token.kind {
                    TokenKind::Identifier(name) => name,
                    _ => {
                        self.errors.push(ErrorContext::new(
                            ErrorCode::Parse005,
                            Some(name_token.row), Some(name_token.col),
                            vec![("token", "インスタンス名")],
                        ));
                        "None".to_string()
                    }
                };

                self.check_next_token(TokenKind::Equal);
                let token = self.peek_token()?;
                match token.kind {
                    TokenKind::Identifier(coroutine_name) => {
                        self.next_token()?;
                        self.check_next_token(TokenKind::LParen);
                        self.check_next_token(TokenKind::RParen);
                        self.check_next_token(TokenKind::Semicolon);
                        Ok(PrivateNode::CoroutineInstantiation { task_name, coroutine_name })
                    },
                    _ => {
                        Err(ErrorContext::new(
                            ErrorCode::Parse005,
                            Some(token.row), Some(token.col),
                            vec![("token", "コルーチン名")],
                        ))
                    },
                }
            },
            _ => {
                Err(ErrorContext::new(
                    ErrorCode::Parse006,
                    Some(row), Some(col),
                    vec![("statement", &keyword.to_string()), ("block", "global block")],
                ))
            }
        }
    }

    /// 引数の構文解析
    fn parse_argument(&mut self) -> Vec<PrivateNode> {

        let mut arguments = Vec::new();
        loop {
            let token = match self.peek_token() {
                Ok(token) => token,
                Err(e) => {
                    self.errors.push(e);
                    arguments.push(PrivateNode::Error);
                    break;
                },
            };
            if token.kind == TokenKind::RParen { break; }

            let arg = match self.parse_assignable() {
                Ok(node) => node,
                Err(e) => {
                    self.errors.push(e);
                    PrivateNode::Error
                },
            };
            arguments.push(arg);

            let token = match self.peek_token() {
                Ok(token) => token,
                Err(e) => {
                    self.errors.push(e);
                    break;
                },
            };
            match token.kind {
                TokenKind::Comma => {
                    match self.next_token() {
                        Ok(_) => {},
                        Err(e) => {
                            self.errors.push(e);
                            break;
                        },
                    };
                },
                TokenKind::RParen => break,
                _ => {
                    self.errors.push(ErrorContext::new(
                        ErrorCode::Parse002,
                        Some(token.row), Some(token.col),
                        vec![("token", &token.kind.to_string())],
                    ));
                    break;
                }
            }
        }

        arguments
    }

    /// 割り当て可能値の構文解析（引数、代入式の右辺）
    fn parse_assignable(&mut self) -> Result<PrivateNode, ErrorContext> {

        let token = self.peek_token()?;
        match token.kind.clone() {
            TokenKind::StringLiteral(_) | TokenKind::NumberLiteral(_) | TokenKind::BoolLiteral(_) 
            | TokenKind::ArithmeticOperator(Arithmetic::Plus|Arithmetic::Minus) | TokenKind::LParen => {
                return self.parse_expression();
            },
            TokenKind::Identifier(name) => {
                let next_token = self.peek_n(1)?;
                match next_token.kind {
                    TokenKind::LParen => {
                        self.next_token()?;
                        self.next_token()?;
                        let arguments = self.parse_argument();
                        
                        self.check_next_token(TokenKind::RParen);
                        
                        return Ok(PrivateNode::FunctionCall {
                            name,
                            arguments,
                            return_flg: true,
                        });
                    },
                    _ => {
                        return self.parse_expression();
                    }
                }
            },
            _ => Err(ErrorContext::new(
                ErrorCode::Parse002,
                Some(token.row), Some(token.col),
                vec![("token", &token.kind.to_string())],
            )),
        }
    }

    /// 式の構文解析
    fn parse_expression(&mut self) -> Result<PrivateNode, ErrorContext> {
        self.parse_logical()
    }

    /// 論理演算の構文解析
    fn parse_logical(&mut self) -> Result<PrivateNode, ErrorContext> {
        self.parse_or_expr()
    }

    /// OR演算の構文解析
    fn parse_or_expr(&mut self) -> Result<PrivateNode, ErrorContext> {
        let mut left = self.parse_and_expr()?;
        loop {
            let token = self.peek_token()?;
            if token.kind != TokenKind::LogicalOperator(Logical::Binary(BinaryLogical::Or)) {
                break;
            }

            self.next_token()?;
            let right = self.parse_and_expr()?;
            left = PrivateNode::Logical {
                operator: Logical::Binary(BinaryLogical::Or),
                left: Box::new(left),
                right: Some(Box::new(right)),
            };
        }
        Ok(left)
    }

    /// AND演算の構文解析
    fn parse_and_expr(&mut self) -> Result<PrivateNode, ErrorContext> {
        let mut left = self.parse_not_expr()?;
        loop {
            let token = self.peek_token()?;
            if let TokenKind::LogicalOperator(op) = token.kind {
                match op {
                    Logical::Binary(BinaryLogical::And) | Logical::Binary(BinaryLogical::Xor) => {
                        self.next_token()?;
                        let right = self.parse_not_expr()?;
                        left = PrivateNode::Logical {
                            operator: op,
                            left: Box::new(left),
                            right: Some(Box::new(right)),
                        };
                    }
                    _ => break,
                }
            }
            else {
                break;
            }
        }
        Ok(left)
    }

    /// NOT演算の構文解析
    fn parse_not_expr(&mut self) -> Result<PrivateNode, ErrorContext> {
        let token = self.peek_token()?;
        match token.kind {
            TokenKind::LogicalOperator(Logical::Unary(UnaryLogical::Not)) => {
                let value = self.parse_expression()?;
                Ok(PrivateNode::Logical {
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
    fn parse_compare(&mut self) -> Result<PrivateNode, ErrorContext> {
        let left = self.parse_value();
        let token = self.peek_token()?;
        let operator = match token.kind {
            TokenKind::CompareOperator(value) => value,
            _ => return left,
        };
        
        self.next_token()?;
        let right = self.parse_value()?;
        return Ok(PrivateNode::Compare {
            operator,
            left: Box::new(left?),
            right: Box::new(right),
        });
    }

    /// 値の構文解析
    fn parse_value(&mut self) -> Result<PrivateNode, ErrorContext> {
        let token = self.peek_token()?;
        match token.kind {
            TokenKind::NumberLiteral(_) | TokenKind::ArithmeticOperator(Arithmetic::Plus|Arithmetic::Minus)
            | TokenKind::LParen | TokenKind::Identifier(_) => {
                return self.parse_add_and_sub()
            },
            TokenKind::StringLiteral(_) => return self.parse_literal(),
            _ => Err(ErrorContext::new(
                ErrorCode::Parse002,
                Some(token.row), Some(token.col),
                vec![("token", &token.kind.to_string())],
            )),
        }
    }

    /// 足し算、引き算の構文解析
    fn parse_add_and_sub(&mut self) -> Result<PrivateNode, ErrorContext> {
        let mut left = self.parse_mul_and_div()?;
        while let Some(TokenKind::ArithmeticOperator(Arithmetic::Plus|Arithmetic::Minus)) = self.tokens.peek().map(|t| &t.kind) {
            let operator = match self.next_token()?.kind {
                TokenKind::ArithmeticOperator(op) => op,
                _ => unreachable!(),
            };
            let right = self.parse_mul_and_div()?;
            left = PrivateNode::Arithmetic {
                operator: operator,
                left: Box::new(left),
                right: Some(Box::new(right)),
            };
        }
        Ok(left)
    }

    /// 掛け算、割り算の構文解析
    fn parse_mul_and_div(&mut self) -> Result<PrivateNode, ErrorContext> {
        let mut left = self.parse_unary()?;
        while let Some(TokenKind::ArithmeticOperator(Arithmetic::Multiply|Arithmetic::Divide)) = self.tokens.peek().map(|t| &t.kind) {
            let operator = match self.next_token()?.kind {
                TokenKind::ArithmeticOperator(op) => op,
                _ => unreachable!(),
            };
            self.next_token()?;
            let right = self.parse_unary()?;
            left = PrivateNode::Arithmetic {
                operator: operator,
                left: Box::new(left),
                right: Some(Box::new(right)),
            };
        }
        Ok(left)
    }

    /// 単項演算子の構文解析
    fn parse_unary(&mut self) -> Result<PrivateNode, ErrorContext> {
        let token = self.peek_token()?;
        match token.kind {
            TokenKind::NumberLiteral(_) | TokenKind::LParen | TokenKind::Identifier(_)
            | TokenKind::ArithmeticOperator(Arithmetic::Plus)=> {
                return self.parse_primary()
            },
            TokenKind::ArithmeticOperator(Arithmetic::Minus) => {
                self.next_token()?;
                let number = self.parse_primary()?;
                Ok(PrivateNode::Arithmetic {
                    operator: Arithmetic::Minus,
                    left: Box::new(number),
                    right: None,
                })
            },
            _ => Err(ErrorContext::new(
                ErrorCode::Parse002,
                Some(token.row), Some(token.col),
                vec![("token", &token.kind.to_string())],
            )),
        }
    }

    /// 数値、計算式の'()'の構文解析
    fn parse_primary(&mut self) -> Result<PrivateNode, ErrorContext> {
        let token = self.peek_token()?;
        match token.kind{
            TokenKind::NumberLiteral(_) => return self.parse_literal(),
            TokenKind::LParen => {
                self.next_token()?;
                let expr = self.parse_add_and_sub();

                self.check_next_token(TokenKind::RParen);
                return expr;
            },
            TokenKind::Identifier(_) => return self.parse_variable(),
            _ => Err(ErrorContext::new(
                ErrorCode::Parse002,
                Some(token.row), Some(token.col),
                vec![("token", &token.kind.to_string())],
            )),
        }
    }

    /// 変数呼び出しの構文解析
    fn parse_variable(&mut self) -> Result<PrivateNode, ErrorContext> {
        let token = self.next_token()?;
        match token.kind {
            TokenKind::Identifier(name) => Ok(PrivateNode::Variable { name }),
            _ => Err(ErrorContext::new(
                ErrorCode::Parse002,
                Some(token.row), Some(token.col),
                vec![("token", &token.kind.to_string())],
            )),
        }
    }

    /// リテラル型の構文解析（String, Number, Bool）
    fn parse_literal(&mut self) -> Result<PrivateNode, ErrorContext> {
        let token = self.next_token()?;
        match token.kind {
            TokenKind::StringLiteral(value) => {
                return Ok(PrivateNode::Literal { value: LiteralValue::String(value) });
            },
            TokenKind::NumberLiteral(integer) => {
                if let Ok(int_value) = integer.parse::<i32>() {
                    let token = self.peek_token()?;
                    match token.kind {
                        TokenKind::Dot => {
                            self.next_token()?;
                            let mut number_string = integer.clone() + ".";
                            let token = self.next_token()?;
                            if let TokenKind::NumberLiteral(decimal) = token.kind {
                                number_string.push_str(&decimal);
                                if let Ok(float_value) = number_string.parse::<f64>() {
                                    return Ok(PrivateNode::Literal { value: LiteralValue::Float(float_value) });
                                }
                            }
                            return Err(ErrorContext::new(
                                ErrorCode::Parse004,
                                Some(token.row), Some(token.col),
                                vec![("number", &number_string)],
                            ))
                        },
                        _ => return Ok(PrivateNode::Literal { value: LiteralValue::Int(int_value) })
                    }
                } else {
                    return Err(ErrorContext::new(
                        ErrorCode::Parse004,
                        Some(token.row), Some(token.col),
                        vec![("number", &integer)],
                    ))
                }
            },
            TokenKind::BoolLiteral(value) => {
                match value {
                    BoolKeyword::True => {
                        return Ok(PrivateNode::Literal { value: LiteralValue::Bool(true) });
                    },
                    BoolKeyword::False => {
                        return Ok(PrivateNode::Literal { value: LiteralValue::Bool(false) });
                    },
                }
            },
            _ => return Err(ErrorContext::new(
                ErrorCode::Parse002,
                Some(token.row), Some(token.col),
                vec![("token", &token.kind.to_string())],
            )),
        }
    }

    fn parse_doc_comment(&mut self, token: &Token) -> Result<Option<PrivateNode>, ErrorContext> {
        if let TokenKind::DocComment(string) = &token.kind {
            self.next_token()?;
            let re = Regex::new(r"@process(.*)").unwrap();
            if let Some(cap) = re.captures(string) {
                let mut process_comment = cap[1].trim().to_string();
                while let TokenKind::DocComment(string) = self.peek_token()?.kind {
                    process_comment = format!("{}\n{}", process_comment, string);
                    self.next_token()?;
                }
                return Ok(Some(PrivateNode::ProcessComment { comment: process_comment }));
            } else {
                let mut doc_comment = string.to_string();
                while let TokenKind::DocComment(string) = self.peek_token()?.kind {
                    doc_comment = format!("{}\n{}", doc_comment, string);
                    self.next_token()?;
                }
                self.doc_comment = doc_comment;
            }
        }
        Ok(None)
    }

    fn get_doc_comment(&mut self) -> Option<String> {
        if self.doc_comment == "" {
            None
        } else {
            Some(self.doc_comment.clone())
        }
    }

    fn peek_token(&mut self) -> Result<Token, ErrorContext> {
        match self.tokens.peek() {
            Some(token) => Ok(token.clone()),
            // self.next_tokenを使ってトークンを進める限りNoneの可能性はない
            None => {
                Err(ErrorContext::new(
                    ErrorCode::Parse003,
                    None, None,
                    vec![],
                ))
            },
        }
    }

    fn next_token(&mut self) -> Result<Token, ErrorContext> {
        match self.tokens.next() {
            // 正常な動作であれば、peekでEOFを確認してループを抜けるため、この条件で問題ない
            Some(token) if token.kind == TokenKind::EOF => {
                Err(ErrorContext::new(
                    ErrorCode::Parse003,
                    Some(token.row), Some(token.col),
                    vec![],
                ))
            },
            Some(token) => Ok(token),
            // このtokenを使ってトークンを進める限りNoneの可能性はない
            None => {
                Err(ErrorContext::new(
                    ErrorCode::Parse003,
                    None, None,
                    vec![],
                ))
            },
        }
    }

    fn check_next_token(&mut self, token_kind: TokenKind) {
        let token = match self.peek_token() {
            Ok(token) => token,
            Err(_) => return,
        };
        if token.kind == token_kind {
            let _ = self.next_token();
        } else {
            let kind_str = &token_kind.to_string();

            self.errors.push(ErrorContext::new(
                ErrorCode::Parse005,
                Some(token.row), Some(token.col),
                vec![("token", kind_str)],
            ));
        }
    }

    /// n+1個先のTokenを確認
    /// 
    /// n=0のとき1個先、n=1のとき2個先
    fn peek_n(&mut self, n: usize) -> Result<Token, ErrorContext> {
        let token = self.peek_token()?;
        match self.tokens.by_ref().clone().nth(n) {
            Some(token) => Ok(token),
            None => Err(ErrorContext::new(
                ErrorCode::Parse003,
                Some(token.row), Some(token.col),
                vec![]
            ))
        }
    }

    fn push_block(&mut self, block_type: BlockType) {
        self.block_stack.push(block_type);
        self.doc_comment = String::new();
    }

    fn pop_block(&mut self) {
        self.block_stack.pop();
        self.doc_comment = String::new();
    }
}

/// 構文解析を行う
pub fn parse(tokens: Vec<Token>) -> (RootNode, Vec<ErrorContext>) {
    let mut parser = Parser::new(tokens);
    let node = parser.parse_program();
    (node, parser.errors)
}