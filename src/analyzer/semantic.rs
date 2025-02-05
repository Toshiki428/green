use crate::{common::types::{LiteralValue, Type}, error::{error_code::ErrorCode, error_context::ErrorContext}, parser::node::*};

use super::{coroutine_table::CoroutineTable, function_table::FunctionTable, task_table::TaskTable, variable_table::VariableTable};

pub struct Semantic {
    pub function_table: FunctionTable,
    pub variable_table: VariableTable,
    pub coroutine_table: CoroutineTable,
    pub task_table: TaskTable,
    pub errors: Vec<ErrorContext>,

    analysis_name: String,
}
impl Semantic {
    fn new() -> Self {
        Self {
            function_table: FunctionTable::new(),
            variable_table: VariableTable::new(),
            coroutine_table: CoroutineTable::new(),
            task_table: TaskTable::new(),
            errors: Vec::new(),

            analysis_name: "".to_string(),
        }
    }

    /// 意味解析処理
    fn semantic(&mut self, ast: &RootNode) {
        let RootNode { functions, coroutines } = ast.clone();

        for FunctionDefinitionNode { name, parameters, return_type, block, doc } in &functions {
            self.function_table.function_definition(name, doc.as_deref(), parameters, return_type, false, block);
        }

        for CoroutineDefinitionNode { name, block, doc} in &coroutines {
            self.coroutine_table.coroutine_definition(name, doc.as_deref(), block);
        }

        for FunctionDefinitionNode { name, parameters:_, return_type:_, block, doc:_ } in functions {
            self.analysis_name = name;
            self.semantic_block(&block);
            self.analysis_name = "".to_string();
        }

        for CoroutineDefinitionNode { name, block, doc:_ } in coroutines {
            self.analysis_name = name;
            self.semantic_block(&block);
            self.analysis_name = "".to_string();
        }
    }

    fn semantic_block(&mut self, block: &BlockNode) {
        for statement in block.statements.clone() {
            self.semantic_statement(&statement);
        }
    }

    fn semantic_statement(&mut self, statement: &PrivateNode) -> Option<Type> {
        match statement {
            PrivateNode::Arithmetic { operator, left, right } => {
                if let Some(right) = right {
                    match self.semantic_binary(&operator.to_string(), &left, &right) {
                        Ok(value_type) => return Some(value_type),
                        Err(_) => return None,
                    }
                } else {
                    match self.semantic_statement(left) {
                        Some(left_type) => return Some(left_type),
                        None => {
                            self.errors.push(
                                ErrorContext::new(
                                    ErrorCode::Semantic003,
                                    None, None,
                                    vec![("node", &format!("{:?}", left))],
                                )
                            );
                            return None
                        },
                    }
                }
            },
            PrivateNode::Break => {},
            PrivateNode::Compare { operator, left, right } => {
                match self.semantic_binary(&operator.to_string(), &left, &right) {
                    Ok(value_type) => return Some(value_type),
                    Err(_) => return None,
                }
            },
            PrivateNode::Continue => {},
            PrivateNode::CoroutineInstantiation { task_name, coroutine_name} => {
                if let Some(coroutine_info) = self.coroutine_table.get_coroutine_info(coroutine_name) {
                    self.task_table.add_task(task_name, coroutine_name, &coroutine_info.process);
                } else {
                    self.errors.push(
                        ErrorContext::new(
                            ErrorCode::Semantic004,
                            None, None,
                            vec![
                                ("statement", "コルーチン"),
                                ("name", coroutine_name),
                            ],
                        )
                    );
                }
            },
            PrivateNode::CoroutineResume { task_name } => {
                self.task_table.get_task(task_name);
            },
            PrivateNode::Error => {},
            PrivateNode::FunctionCall { name, arguments, return_flg } => {
                match self.function_table.get_function_info(&name) {
                    Some(function_info) => {
                        if !function_info.is_variadic && function_info.parameters.len() != arguments.len() {
                            self.errors.push(
                                ErrorContext::new(
                                    ErrorCode::Semantic008,
                                    None, None, 
                                    vec![
                                        ("parameter", &function_info.parameters.len().to_string()),
                                        ("argument", &arguments.len().to_string()),
                                        ("name", name),
                                    ],
                                )
                            );
                            return None
                        }

                        if !function_info.is_variadic{
                            for (param, arg) in function_info.parameters.iter().zip(arguments) {
                                if let Some(arg_type) = self.semantic_statement(&arg) {
                                    if arg_type != param.variable_type {
                                        self.errors.push(
                                            ErrorContext::new(
                                                ErrorCode::Semantic006,
                                                None, None,
                                                vec![
                                                    ("variable_name", &param.name),
                                                    ("variable_type", &param.variable_type.to_string()),
                                                    ("value_type", &arg_type.to_string()),
                                                ],
                                            )
                                        );
                                        return None
                                    }
                                } else {
                                    self.errors.push(
                                        ErrorContext::new(
                                            ErrorCode::Semantic006,
                                            None, None,
                                            vec![
                                                ("variable_name", &param.name),
                                                ("variable_type", &param.variable_type.to_string()),
                                                ("value_type", "None"),
                                            ],
                                        )
                                    );
                                    return None
                                }
                            }
                        }

                        if *return_flg {
                            match function_info.return_type {
                                Some(return_type) => return Some(return_type),
                                None => {
                                    self.errors.push(
                                        ErrorContext::new(
                                            ErrorCode::Semantic005, 
                                            None, None,
                                            vec![("function_name", &name)],
                                        )
                                    );
                                    return None
                                }
                            }
                        }
                    },
                    None => {
                        self.errors.push(
                            ErrorContext::new(
                                ErrorCode::Semantic004,
                                None, None,
                                vec![("function_name", &name)],
                            )
                        );
                    },
                }
                return None
            },
            PrivateNode::IfStatement { condition_node:_, then_block, else_block } => {
                self.semantic_block(then_block);
                if let Some(else_block) = else_block {
                    self.semantic_block(else_block);
                }
            },
            PrivateNode::Literal { value } => {
                match value {
                    LiteralValue::Bool(_) => return Some(Type::Bool),
                    LiteralValue::Float(_) => return Some(Type::Float),
                    LiteralValue::Int(_) => return Some(Type::Int),
                    LiteralValue::String(_) => return Some(Type::String),
                    LiteralValue::Null => return None,
                }
            },
            PrivateNode::Logical { operator, left, right } => {
                if let Some(right) = right {
                    match self.semantic_binary(&operator.to_string(), &left, &right) {
                        Ok(value_type) => return Some(value_type),
                        Err(_) => return None,
                    }
                } else {
                    match self.semantic_statement(left) {
                        Some(left_type) => return Some(left_type),
                        None => {
                            self.errors.push(
                                ErrorContext::new(
                                    ErrorCode::Semantic003,
                                    None, None,
                                    vec![("node", &format!("{:?}", left))],
                                )
                            );
                            return None
                        },
                    }
                }
            },
            PrivateNode::LoopStatement { condition_node:_, block } => {
                self.semantic_block(block);
            },
            PrivateNode::ProcessComment { comment:_ } => {},
            PrivateNode::ReturnStatement { assignalbe:_ } => {},
            PrivateNode::Variable { name } => {
                let function_info = match self.function_table.get_function_info(&self.analysis_name) {
                    Some(function_info) => function_info,
                    None => panic!("解析中の関数が存在しない"),
                };

                match function_info.local_variables.get_type(name) {
                    Some(variable_type) => return Some(variable_type),
                    None => {
                        self.errors.push(
                            ErrorContext::new(
                                ErrorCode::Semantic007,
                                None, None,
                                vec![("variable_name", name)],
                            )
                        );
                        return None
                    }
                }
            },
            PrivateNode::VariableAssignment { name, expression } => {
                let function_info = match self.function_table.get_function_info(&self.analysis_name) {
                    Some(function_info) => function_info,
                    None => panic!("解析中の関数が存在しない"),
                };
                
                let variable_type = match function_info.local_variables.get_type(name) {
                    Some(variable_type) => variable_type,
                    None => {
                        self.errors.push(
                            ErrorContext::new(
                                ErrorCode::Semantic007,
                                None, None,
                                vec![("variable_name", name)],
                            )
                        );
                        return None
                    },
                };

                match self.semantic_statement(&expression) {
                    Some(value_type) if value_type == variable_type => {
                        return Some(value_type)
                    },
                    Some(value_type) => {
                        self.errors.push(
                            ErrorContext::new(
                                ErrorCode::Semantic006,
                                None, None,
                                vec![
                                    ("variable_name", &name),
                                    ("variable_type", &variable_type.to_string()),
                                    ("value_type", &value_type.to_string()),
                                ],
                            )
                        );
                        return None
                    },
                    None => {
                        self.errors.push(
                            ErrorContext::new(
                                ErrorCode::Semantic006,
                                None, None,
                                vec![
                                    ("variable_name", &name),
                                    ("variable_type", &variable_type.to_string()),
                                    ("value_type", "none"),
                                ],
                            )
                        );
                        return None
                    },
                }
            },
            PrivateNode::VariableDeclaration { name, variable_type, initializer, doc:_ } => {
                if let Some(function_info) = self.function_table.get_function_info_mut(&self.analysis_name) {
                    function_info.local_variables.variable_declare(name, variable_type);
                } else if let Some(coroutine_info) = self.coroutine_table.get_coroutine_info_mut(&self.analysis_name) {
                    coroutine_info.local_variables.variable_declare(name, variable_type);
                } else {
                    println!("関数名: {}", self.analysis_name);
                    panic!("解析中の関数が存在しない");
                };
                
                if let Some(node) = initializer {
                    match self.semantic_statement(node) {
                        Some(value_type) if &value_type == variable_type => {
                            return Some(variable_type.clone())
                        },
                        Some(value_type) => {
                            self.errors.push(
                                ErrorContext::new(
                                    ErrorCode::Semantic006,
                                    None, None,
                                    vec![
                                        ("variable_name", &name),
                                        ("variable_type", &variable_type.to_string()),
                                        ("value_type", &value_type.to_string()),
                                    ],
                                )
                            );
                            return None
                        },
                        None => {
                            self.errors.push(
                                ErrorContext::new(
                                    ErrorCode::Semantic006,
                                    None, None,
                                    vec![
                                        ("variable_name", &name),
                                        ("variable_type", &variable_type.to_string()),
                                        ("value_type", "none"),
                                    ],
                                )
                            );
                            return None
                        }
                    }
                }
            },
            PrivateNode::Yield => {},
        }
        None
    }

    fn semantic_binary(&mut self, operator:&str, left: &PrivateNode, right: &PrivateNode) -> Result<Type, ()> {
        let left_type = match self.semantic_statement(left) {
            Some(left_type) => left_type,
            None => {
                self.errors.push(
                    ErrorContext::new(
                        ErrorCode::Semantic003,
                        None, None,
                        vec![("node", &format!("{:?}", left))],
                    )
                );
                return Err(())
            }
        };

        let right_type = match self.semantic_statement(right) {
            Some(right_type) => right_type,
            None => {
                self.errors.push(
                    ErrorContext::new(
                        ErrorCode::Semantic003,
                        None, None,
                        vec![("node", &format!("{:?}", right))],
                    )
                );
                return Err(())
            }
        };

        if left_type != right_type {
            self.errors.push(
                ErrorContext::new(
                    ErrorCode::Semantic002,
                    None, None,
                    vec![
                        ("left", &left_type.to_string()),
                        ("operator", operator),
                        ("right", &right_type.to_string()),
                    ],
                )
            );
            return Err(())
        }

        Ok(left_type)
    }
}

pub fn semantic(ast: &RootNode) -> Result<Semantic, Vec<ErrorContext>> {
    let mut semantic = Semantic::new();
    semantic.semantic(&ast);

    if semantic.errors.is_empty() {
        return Ok(semantic)
    }
    else {
        return Err(semantic.errors)
    }
}