use std::collections::HashMap;

use crate::{operator::{ Arithmetic, BinaryArithmetic, BinaryLogical, Comparison, Logical, UnaryArithmetic, UnaryLogical}, parser::Node, types::{LiteralValue, Type, GreenValue}, utils};

#[derive(Debug)]
struct Environment {
    scopes: Vec<HashMap<String, GreenValue>>,
}

impl Environment {
    fn new() -> Self {
        Environment {
            scopes: vec![HashMap::new()],
        }
    }

    fn set_variable(&mut self, name: &String, value: &GreenValue) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.clone(), value.clone());
        }
    }

    fn get_variable(&mut self, name: &str) -> Result<LiteralValue, String> {
        for scope in self.scopes.iter().rev() {
            if let Some(variable) = scope.get(name) {
                return Ok(variable.value.clone());
            }
        }
        Err(utils::get_error_message("RUNTIME", &[("variable", name)])?)
    }

    fn change_variable(&mut self, name: String, value: GreenValue) -> Result<(), String> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(variable) = scope.get_mut(&name) {
                if variable.value_type == value.value_type {
                    variable.value = value.value;
                    return Ok(())
                } else {
                    return Err(format!(
                        "型エラー: 変数 '{}' は {} なのに {} を代入しようとしています",
                        name, variable.value_type.to_string(), value.value_type.to_string()
                    ));
                }
            }
        }
        Err(utils::get_error_message("RUNTIME007", &[("variable", &name)])?)
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }
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
    fn execute(&mut self, node: &Node) -> Result<(), String> {
        match &node {
            Node::Program { statements } => {
                for child in statements {
                    self.statement(child)?;
                }
            },
            _ => return Err(utils::get_error_message("RUNTIME003", &[])?),
        }
        Ok(())
    }

    fn statement(&mut self, node: &Node) -> Result<(), String> {
        match &node {
            Node::FunctionCall { name: _, arguments: _ } => self.execute_function(node)?,
            Node::VariableDeclaration { name, variable_type } => {
                let value = GreenValue {
                    value_type: variable_type.clone(),
                    value: LiteralValue::Null,
                };
                self.variables.set_variable(name, &value);
            },
            Node::VariableAssignment { name, expression } => {
                let value = self.evaluate_assignable(expression)?;
                self.variables.change_variable(name.to_string(), value)?;
            },
            Node::IfStatement { condition_node, then_block, else_block } => {
                self.evaluate_if_statement(condition_node, then_block, else_block)?
            },
            Node::FunctionDefinition { name, parameters, block } => {
                let mut variables = Vec::new();
                for param in parameters {
                    match param {
                        Node::VariableDeclaration { name, variable_type } => {
                            variables.push((name.to_string(), variable_type.clone()));
                        },
                        _ => return Err("謎の引数".to_string()),
                    }
                }
                self.functions.insert(name.to_string(), (variables, block.clone()));
            },
            _ => return Err(utils::get_error_message("RUNTIME003", &[])?),
        }
        Ok(())
    }

    /// print関数の実行
    fn print_function(&mut self, arguments: &Vec<Node>) -> Result<(), String> {
        let values = self.evaluate_argument(arguments)?;
        let result = values.iter().map(|x| x.value.to_string()).collect::<Vec<_>>().join(" ");
        println!("{}", result);
        Ok(())
    }

    fn execute_function(&mut self, node: &Node) -> Result<(), String> {
        match &node {
            Node::FunctionCall { name, arguments } => {
                match name.as_str() {
                    "print" => self.print_function(arguments)?,
                    _ => {
                        let function_data = self.functions.get(name).cloned();
                        if let Some((parameters, function_node)) = function_data {
                            if parameters.len() != arguments.len() {
                                return Err(format!(
                                    "関数 {} の引数の数が一致しません (期待: {}, 受け取った: {})", 
                                    name, parameters.len(), arguments.len()
                                ));
                            }

                            self.variables.push_scope();

                            let values = self.evaluate_argument(arguments)?;
                            for ((name, param), value) in parameters.into_iter().zip(values.into_iter()) {
                                if param == value.value_type {
                                    self.variables.set_variable(&name, &value);
                                }
                                else {
                                    return Err(format!(
                                        "関数 {} の引数の型が一致しません (期待: {}, 受け取った: {})",
                                        name, param.to_string(), value.value_type.to_string(),
                                    ))
                                }
                            }

                            self.execute(&function_node)?;
                        } else {
                            return Err(utils::get_error_message("RUNTIME002", &[("function", name)])?);
                        }

                        self.variables.pop_scope();
                    },
                }
                Ok(())
            }
            _ => Err("Invalid function node".to_string()),
        }
    }

    fn evaluate_if_statement(&mut self, condition_node: &Node, then_block: &Node, else_block: &Option<Box<Node>>) -> Result<(), String> {
        if let LiteralValue::Bool(condition_result) = self.evaluate_assignable(&condition_node)?.value {
            if condition_result {
                self.execute(then_block)?;
            } else {
                if let Some(else_node) = else_block {
                    self.execute(else_node)?;
                }
            }
        } else {
            return Err(format!("if文の条件が正しくない"))
        }
        Ok(())
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
            Node::Literal{ value: _ } => self.evaluate_literal(node)?,
            _ => return Err(utils::get_error_message("RUNTIME005", &[])?),
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
                            Err(format!("想定外の論理演算: 演算子: {:?} 値: {:?}", unary_operator, node))
                        }
                    },
                    Logical::Binary(binary_operator) => {
                        let left = self.evaluate_expression(left)?;
                        let right = match right {
                            Some(right) => self.evaluate_expression(&right)?,
                            None => return Err(utils::get_error_message("RUNTIME009", &[])?),
                        };
                        match (left, right) {
                            (LiteralValue::Bool(left_value), LiteralValue::Bool(right_value)) => {
                                let result = self.binary_logical_operations(binary_operator, left_value, right_value)?;
                                Ok(LiteralValue::Bool(result))
                            },
                            (left_value, right_value) => {
                                Err(format!("想定外の論理演算: 左: {:?} 演算子: {:?} 右: {:?}", left_value, binary_operator, right_value))
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
                            _ => Err(utils::get_error_message("RUNTIME006", &[("operator", format!("{:?}", operator).as_str())])?),
                        }
                    },
                    (left_value, right_value) => {
                        Err(format!("異なる型の比較はできません: 左: {:?} 右: {:?}", left_value, right_value))
                    },
                }
            },
            // 四則演算
            Node::Arithmetic {
                operator: Arithmetic::Binary(binary_operator),
                left,
                right
            } => {
                let right = if let Some(right_node) = right {
                    right_node
                } else {
                    return Err("右辺がない".to_string());
                };
                let left_literal = self.evaluate_expression(left)?;
                let right_literal = self.evaluate_expression(right)?;
                match (&left_literal, &right_literal) {
                    (LiteralValue::Int(left_value), LiteralValue::Int(right_value)) => {
                        match binary_operator {
                            BinaryArithmetic::Add => Ok(LiteralValue::Int(left_value + right_value)),
                            BinaryArithmetic::Subtract => Ok(LiteralValue::Int(left_value - right_value)),
                            BinaryArithmetic::Multiply => Ok(LiteralValue::Int(left_value * right_value)),
                            BinaryArithmetic::Divide => Ok(LiteralValue::Int(left_value / right_value)),
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
                        match binary_operator {
                            BinaryArithmetic::Add => Ok(LiteralValue::Float(left_value + right_value)),
                            BinaryArithmetic::Subtract => Ok(LiteralValue::Float(left_value - right_value)),
                            BinaryArithmetic::Multiply => Ok(LiteralValue::Float(left_value * right_value)),
                            BinaryArithmetic::Divide => Ok(LiteralValue::Float(left_value / right_value)),
                        }
                    },
                    _ => Err(format!("想定外のAddAndSub型: {:?}", node))
                }
            },

            Node::Arithmetic {
                operator: Arithmetic::Unary(unary_operator),
                left,
                right: _,
            } => {
                match self.evaluate_expression(left)? {
                    LiteralValue::Float(value) => {
                        let mut result = value;
                        if *unary_operator == UnaryArithmetic::Minus {
                            result = -1.0 * value;
                        }
                        Ok(LiteralValue::Float(result))
                    },
                    LiteralValue::Int(value) => {
                        let mut result = value;
                        if *unary_operator == UnaryArithmetic::Minus {
                            result = -1 * value;
                        }
                        Ok(LiteralValue::Int(result))
                    },
                    _ => Err(format!("想定外のPrimary型: {:?}", left)),
                }
            },
            Node::Variable { name } => {
                let variable = self.variables.get_variable(name)?;
                Ok(variable)
            },
            Node::Literal { value: _ } => self.evaluate_literal(node),
            _ => Err(utils::get_error_message("RUNTIME003", &[])?),
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