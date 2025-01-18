use std::collections::HashMap;

use crate::{
    common::{
        operator::{ Arithmetic, BinaryLogical, Comparison, Logical, UnaryLogical},
        types::{GreenValue, LiteralValue, Type},
    },
    interpreter::environment::Environment,
    parser::node::Node,
    error::{
        error_message::ErrorMessage,
        error_code::ErrorCode,
        error_context::ErrorContext,
    },
};

#[derive(Debug)]
pub enum EvalFlow<T> {
    Normal,
    Continue,
    Break,
    Return(T),
}

struct Interpreter {
    variables: Environment,
    functions: HashMap<String, (Vec<(String, Type)>, Box<Node>)>,
}

impl Interpreter {
    fn new() -> Self {
        Interpreter {
            variables: Environment::new(),
            functions: HashMap::new(),
        }
    }

    /// プログラムの実行
    fn execute(&mut self, node: &Node) -> Result<EvalFlow<GreenValue>, String> {
        match &node {
            Node::Block { block_type:_, statements } => {
                for child in statements {
                    let result = self.statement(child)?;
                    match result {
                        EvalFlow::Return(_) | EvalFlow::Break | EvalFlow::Continue => return Ok(result),
                        EvalFlow::Normal => {},
                    }
                }
            },
            _ => return Err(ErrorMessage::global().get_error_message(
                &ErrorContext::new(
                    ErrorCode::Runtime003,
                    0, 0,
                    vec![("node", &format!("{:?}",node))],
                )
            )?),
        }
        Ok(EvalFlow::Normal)
    }

    fn statement(&mut self, node: &Node) -> Result<EvalFlow<GreenValue>, String> {
        match &node {
            Node::FunctionCall { name: _, arguments: _ } => {
                self.execute_function(node)?;
            },
            Node::VariableDeclaration { name, variable_type, initializer } => {
                let value_type = Type::from_keyword(variable_type);
                let value = match initializer {
                    Some(expression) => self.evaluate_assignable(expression)?.value,
                    None => LiteralValue::Null,
                };

                let value = GreenValue {
                    value_type,
                    value,
                };
                self.variables.set_variable(name, &value);
            },
            Node::VariableAssignment { name, expression } => {
                let value = self.evaluate_assignable(expression)?;
                self.variables.change_variable(name.to_string(), value)?;
            },
            Node::IfStatement { condition_node, then_block, else_block } => {
                return self.evaluate_if_statement(condition_node, then_block, else_block);
            },
            Node::LoopStatement { condition_node, block } => {
                return self.evaluate_loop_statement(condition_node, block);
            },
            Node::FunctionDefinition { name, parameters, block } => {
                let mut variables = Vec::new();
                for param in parameters {
                    match param {
                        Node::VariableDeclaration { name, variable_type, initializer:_ } => {
                            variables.push((name.to_string(), Type::from_keyword(variable_type)));
                        },
                        _ => return Err(ErrorMessage::global().get_error_message(
                            &ErrorContext::new(
                                ErrorCode::Runtime011,
                                0, 0,
                                vec![("node", &format!("{:?}", node))],
                            )
                        )?),
                    }
                }
                self.functions.insert(name.to_string(), (variables, block.clone()));
            },
            Node::ReturnStatement { assignalbe } => {
                let return_value = self.evaluate_assignable(assignalbe)?;
                return Ok(EvalFlow::Return(return_value));
            },
            Node::Break => {
                return Ok(EvalFlow::Break);
            },
            Node::Continue => {
                return Ok(EvalFlow::Continue);
            }
            _ => return Err(ErrorMessage::global().get_error_message(
                &ErrorContext::new(
                    ErrorCode::Runtime003,
                    0, 0,
                    vec![("node", &format!("{:?}", node))],
                )
            )?),
        }
        Ok(EvalFlow::Normal)
    }

    /// print関数の実行
    fn print_function(&mut self, arguments: &Vec<Node>) -> Result<(), String> {
        let values = self.evaluate_argument(arguments)?;
        let result = values.iter().map(|x| x.value.to_string()).collect::<Vec<_>>().join(" ");
        println!("{}", result);
        Ok(())
    }

    fn execute_function(&mut self, node: &Node) -> Result<Option<GreenValue>, String> {
        match &node {
            Node::FunctionCall { name, arguments } | Node::FunctionCallWithReturn { name, arguments } => {
                match name.as_str() {
                    "print" => self.print_function(arguments)?,
                    _ => {
                        let function_data = self.functions.get(name).cloned();
                        if let Some((parameters, function_node)) = function_data {
                            if parameters.len() != arguments.len() {
                                return Err(ErrorMessage::global().get_error_message(
                                    &ErrorContext::new(
                                        ErrorCode::Runtime012, 
                                        0, 0,
                                        vec![
                                            ("parameters", &parameters.len().to_string()),
                                            ("arguments", &arguments.len().to_string()),
                                            ("name", name),
                                        ],
                                    )
                                )?);
                            }
                            self.variables.push_scope();

                            let values = self.evaluate_argument(arguments)?;
                            for ((param_name, param_type), value) in parameters.into_iter().zip(values.into_iter()) {
                                if param_type == value.value_type {
                                    self.variables.set_variable(&param_name, &value);
                                }
                                else {
                                    return Err(ErrorMessage::global().get_error_message(
                                        &ErrorContext::new(
                                            ErrorCode::Runtime013,
                                            0, 0,
                                            vec![
                                                ("parameter", &param_type.to_string()),
                                                ("argument", &value.value_type.to_string()),
                                                ("function_name", name),
                                                ("param_name", &param_name),
                                            ],
                                        )
                                    )?)
                                }
                            }

                            let result = self.execute(&function_node)?;
                            match result {
                                EvalFlow::Return(value) => {
                                    self.variables.pop_scope();
                                    return Ok(Some(value))
                                },
                                EvalFlow::Normal => {},
                                _ => {
                                    return Err(ErrorMessage::global().get_error_message(
                                        &ErrorContext::new(
                                            ErrorCode::Runtime018,
                                            0, 0,
                                            vec![("node", &format!("{:?}", result))],
                                        )
                                    )?)
                                }
                            }
                        } else {
                            return Err(ErrorMessage::global().get_error_message(
                                &ErrorContext::new(
                                    ErrorCode::Runtime002,
                                    0, 0,
                                    vec![("function", name)],
                                )
                            )?);
                        }

                        self.variables.pop_scope();
                    },
                }
                Ok(None)
            }
            _ => return Err(ErrorMessage::global().get_error_message(
                &ErrorContext::new(
                    ErrorCode::Runtime003,
                    0, 0,
                    vec![("node", &format!("{:?}", node))],
                )
            )?),
        }
    }

    fn evaluate_if_statement(&mut self, condition_node: &Node, then_block: &Node, else_block: &Option<Box<Node>>) -> Result<EvalFlow<GreenValue>, String> {
        if let LiteralValue::Bool(condition_result) = self.evaluate_assignable(&condition_node)?.value {
            match condition_result {
                true => return self.execute(then_block),
                false => {
                    if let Some(else_node) = else_block {
                        return self.execute(else_node)
                    }
                }
            }
        } else {
            return Err(ErrorMessage::global().get_error_message(
                &ErrorContext::new(
                    ErrorCode::Runtime014,
                    0, 0,
                    vec![("node", &format!("{:?}",condition_node))],
                )
            )?)
        }
        Ok(EvalFlow::Normal)
    }

    fn evaluate_loop_statement(&mut self, condition_node: &Node, block: &Node) -> Result<EvalFlow<GreenValue>, String> {
        loop {
            let condition_value = self.evaluate_assignable(&condition_node)?;

            match condition_value.value {
                LiteralValue::Bool(true) => {
                    let result = self.execute(block)?;
                    match result {
                        EvalFlow::Break => break,
                        EvalFlow::Continue => continue,
                        EvalFlow::Return(_) => return Ok(result),
                        EvalFlow::Normal => {},
                    }
                },
                LiteralValue::Bool(false) => break,
                _ => return Err(ErrorMessage::global().get_error_message(
                    &ErrorContext::new(
                        ErrorCode::Runtime014,
                        0, 0,
                        vec![("node", &format!("{:?}",condition_node))],
                    )
                )?)
            }
        }
        Ok(EvalFlow::Normal)
    }

    /// 引数の評価
    fn evaluate_argument(&mut self, arguments: &Vec<Node>) -> Result<Vec<GreenValue>, String> {
        let mut values = Vec::new();
        for child in arguments {
            values.push(self.evaluate_assignable(child)?);
        }
        return Ok(values)
    }

    /// 割り当て可能値の評価（引数、代入式の右辺など）
    /// LiteralValueからGreenValueへの変換も行う
    fn evaluate_assignable(&mut self, node: &Node) -> Result<GreenValue, String> {
        let literal_value = match &node {
            Node::Compare{ operator: _, left: _, right: _ } | Node::Arithmetic{ operator: _, left: _, right: _ }
            | Node::Variable { name: _ } | Node::Logical{ operator: _, left: _, right: _ } => {
                self.evaluate_expression(node)?
            },
            Node::FunctionCallWithReturn { name:_, arguments:_ } => {
                self.execute_function(node)?.ok_or("err")?.value
            },
            Node::Literal{ value: _ } => self.evaluate_literal(node)?,
            _ => return Err(ErrorMessage::global().get_error_message(
                &ErrorContext::new(
                    ErrorCode::Runtime005,
                    0, 0,
                    vec![("node", &format!("{:?}", node))],
                )
            )?),
        };

        match literal_value {
            LiteralValue::Bool(_) => Ok(GreenValue::new(Type::Bool, literal_value)),
            LiteralValue::Float(_) => Ok(GreenValue::new(Type::Float, literal_value)),
            LiteralValue::Int(_) => Ok(GreenValue::new(Type::Int, literal_value)),
            LiteralValue::String(_) => Ok(GreenValue::new(Type::String, literal_value)),
            LiteralValue::Null => Err("Null値には未対応".to_string()),
        }
    }

    /// 式の評価
    fn evaluate_expression(&mut self, node: &Node) -> Result<LiteralValue, String> {
        match &node {
            // 論理演算
            Node::Logical {
                operator,
                left,
                right,
            } => {
                match operator {
                    Logical::Unary(unary_operator) => {
                        let node = left;
                        if let LiteralValue::Bool(value)  = self.evaluate_expression(node)? {
                            let result = self.unary_logical_operations(unary_operator, value)?;
                            Ok(LiteralValue::Bool(result))
                        } else {
                            Err(ErrorMessage::global().get_error_message(
                                &ErrorContext::new(
                                    ErrorCode::Runtime008,
                                    0, 0,
                                    vec![
                                        ("operator", &operator.to_string()),
                                        ("node", &format!("{:?}", node)),
                                    ],
                                )
                            )?)
                        }
                    },
                    Logical::Binary(binary_operator) => {
                        let left = self.evaluate_expression(left)?;
                        let right = match right {
                            Some(right) => self.evaluate_expression(&right)?,
                            None => return Err(ErrorMessage::global().get_error_message(
                                &ErrorContext::new(
                                    ErrorCode::Runtime009,
                                    0, 0,
                                    vec![],
                                )
                            )?),
                        };
                        match (left, right) {
                            (LiteralValue::Bool(left_value), LiteralValue::Bool(right_value)) => {
                                let result = self.binary_logical_operations(binary_operator, left_value, right_value)?;
                                Ok(LiteralValue::Bool(result))
                            },
                            (left_value, right_value) => {
                                Err(ErrorMessage::global().get_error_message(
                                    &ErrorContext::new(
                                        ErrorCode::Runtime015,
                                        0, 0,
                                        vec![
                                            ("left", &left_value.to_string()),
                                            ("operator", &operator.to_string()),
                                            ("right", &right_value.to_string()),
                                        ],
                                    )
                                )?)
                            },
                        }
                    },
                }
            },
            // 比較
            Node::Compare {
                operator,
                left,
                right
            } => {
                let left = self.evaluate_expression(left)?;
                let right = self.evaluate_expression(right)?;
                match (left.clone(), right.clone()) {
                    (LiteralValue::Float(_), LiteralValue::Float(_)) | 
                    (LiteralValue::Int(_), LiteralValue::Int(_)) |
                    (LiteralValue::Float(_), LiteralValue::Int(_)) |
                    (LiteralValue::Int(_), LiteralValue::Float(_)) => {
                        let left_value = match left {
                            LiteralValue::Float(value) => value,
                            LiteralValue::Int(value) => value as f64,
                            _ => unreachable!(),
                        };
                        let right_value = match right {
                            LiteralValue::Float(value) => value,
                            LiteralValue::Int(value) => value as f64,
                            _ => unreachable!(),
                        };
                        let result = self.compare_values(operator, left_value, right_value)?;
                        Ok(LiteralValue::Bool(result))
                    },
                    (LiteralValue::String(left_value), LiteralValue::String(right_value)) => {
                        match operator {
                            Comparison::Equal => Ok(LiteralValue::Bool(left_value == right_value)),
                            Comparison::NotEqual => Ok(LiteralValue::Bool(left_value != right_value)),
                            _ => Err(ErrorMessage::global().get_error_message(
                                &ErrorContext::new(
                                    ErrorCode::Runtime006,
                                    0, 0,
                                    vec![("operator", &operator.to_string())],
                                )
                            )?),
                        }
                    },
                    (left_value, right_value) => {
                        Err(ErrorMessage::global().get_error_message(
                            &ErrorContext::new(
                                ErrorCode::Runtime016,
                                0, 0,
                                vec![
                                    ("left", &left_value.to_string()),
                                    ("operator", &operator.to_string()),
                                    ("right", &right_value.to_string()),
                                ],
                            )
                        )?)
                    },
                }
            },
            // 四則演算
            Node::Arithmetic {
                operator,
                left,
                right
            } => {
                match right {
                    Some(right) => {
                        let left_literal = self.evaluate_expression(left)?;
                        let right_literal = self.evaluate_expression(right)?;
                        match (&left_literal, &right_literal) {
                            (LiteralValue::Int(left_value), LiteralValue::Int(right_value)) => {
                                match operator {
                                    Arithmetic::Plus => Ok(LiteralValue::Int(left_value + right_value)),
                                    Arithmetic::Minus => Ok(LiteralValue::Int(left_value - right_value)),
                                    Arithmetic::Multiply => Ok(LiteralValue::Int(left_value * right_value)),
                                    Arithmetic::Divide => Ok(LiteralValue::Int(left_value / right_value)),
                                }
                            },
                            (LiteralValue::Int(_), LiteralValue::Float(_)) |
                            (LiteralValue::Float(_), LiteralValue::Int(_)) |
                            (LiteralValue::Float(_), LiteralValue::Float(_)) => {
                                let left_value = match left_literal {
                                    LiteralValue::Float(value) => value,
                                    LiteralValue::Int(value) => value as f64,
                                    _ => unreachable!(),
                                };
                                let right_value = match right_literal {
                                    LiteralValue::Float(value) => value,
                                    LiteralValue::Int(value) => value as f64,
                                    _ => unreachable!(),
                                };
                                match operator {
                                    Arithmetic::Plus => Ok(LiteralValue::Float(left_value + right_value)),
                                    Arithmetic::Minus => Ok(LiteralValue::Float(left_value - right_value)),
                                    Arithmetic::Multiply => Ok(LiteralValue::Float(left_value * right_value)),
                                    Arithmetic::Divide => Ok(LiteralValue::Float(left_value / right_value)),
                                }
                            },
                            _ => Err(ErrorMessage::global().get_error_message(
                                &ErrorContext::new(
                                    ErrorCode::Runtime015,
                                    0, 0,
                                    vec![
                                        ("left", &left_literal.to_string()),
                                        ("operator", &operator.to_string()),
                                        ("right", &right_literal.to_string()),
                                    ],
                                )
                            )?)
                        }
                    },
                    None => {
                        match operator {
                            Arithmetic::Plus | Arithmetic::Minus => {
                                match self.evaluate_expression(left)? {
                                    LiteralValue::Float(value) => {
                                        let mut result = value;
                                        if *operator == Arithmetic::Minus {
                                            result = -1.0 * value;
                                        }
                                        Ok(LiteralValue::Float(result))
                                    },
                                    LiteralValue::Int(value) => {
                                        let mut result = value;
                                        if *operator == Arithmetic::Minus {
                                            result = -1 * value;
                                        }
                                        Ok(LiteralValue::Int(result))
                                    },
                                    _ => Err(ErrorMessage::global().get_error_message(
                                        &ErrorContext::new(
                                            ErrorCode::Runtime008,
                                            0, 0,
                                            vec![
                                                ("operator", &operator.to_string()),
                                                ("node", &format!("{:?}", left)),
                                            ],
                                        )
                                    )?),
                                }
                            },
                            _ => {
                                return Err(ErrorMessage::global().get_error_message(
                                    &ErrorContext::new(
                                        ErrorCode::Runtime009,
                                        0, 0,
                                        vec![],
                                    )
                                )?);
                            }
                        }
                    },
                }
            },
            Node::Variable { name } => {
                let variable = self.variables.get_variable(name)?;
                Ok(variable)
            },
            Node::Literal { value: _ } => self.evaluate_literal(node),
            _ => Err(ErrorMessage::global().get_error_message(
                &ErrorContext::new(
                    ErrorCode::Runtime003,
                    0, 0,
                    vec![("node", &format!("{:?}",node))],
                )
            )?),
        }
    }

    /// 単項論理演算
    fn unary_logical_operations(&mut self, operator: &UnaryLogical, operand: bool) -> Result<bool, String> {
        match operator {
            UnaryLogical::Not => Ok(!operand),
        }
    }

    /// 二項論理演算子
    fn binary_logical_operations(&mut self, operator: &BinaryLogical, left: bool, right: bool) -> Result<bool, String> {
        match operator {
            BinaryLogical::Or => Ok(left || right),
            BinaryLogical::And => Ok(left && right),
            BinaryLogical::Xor => Ok(left && !right || !left && right),
        }
    }

    /// 比較処理
    fn compare_values(&mut self, operator: &Comparison, left: f64, right: f64) -> Result<bool, String> {
        match operator {
            Comparison::Equal => Ok(left == right),
            Comparison::NotEqual => Ok(left != right),
            Comparison::GreaterEqual => Ok(left >= right),
            Comparison::Greater => Ok(left > right),
            Comparison::LessEqual => Ok(left <= right),
            Comparison::Less => Ok(left < right),
        }
    }

    /// 値の評価
    fn evaluate_literal(&mut self, node: &Node) -> Result<LiteralValue, String> {
        match &node {
            Node::Literal { value } => Ok(value.clone()),
            _ => { Err(format!("想定外のリテラルの型: {:?}", node)) }
        }
    }

}

pub fn execute(node: &Node) -> Result<(), String> {
    let mut interpreter = Interpreter::new();
    interpreter.execute(node)?;
    Ok(())
}