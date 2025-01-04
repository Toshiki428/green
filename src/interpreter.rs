use std::collections::HashMap;

use crate::{operator::{ Arithmetic, BinaryArithmetic, BinaryLogical, Comparison, Logical, UnaryArithmetic, UnaryLogical}, parser::{LiteralValue, Node, NodeKind}, utils};

#[derive(Debug, Clone)]
enum GreenType {
    Float(f64),
    // Int(i32),
    Bool(bool),
    String(String),
}

impl ToString for GreenType {
    fn to_string(&self) -> String {
        match self {
            // GreenType::Int(i) => i.to_string(),
            GreenType::Float(f) => f.to_string(),
            GreenType::String(s) => s.clone(),
            GreenType::Bool(b) => b.to_string(),
        }
    }
}

#[derive(Debug)]
struct Environment {
    scopes: Vec<HashMap<String, GreenType>>,
}

impl Environment {
    fn new() -> Self {
        Environment {
            scopes: vec![HashMap::new()],
        }
    }

    fn set_variable(&mut self, name: String, value: GreenType) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    fn get_variable(&mut self, name: &str) -> Result<GreenType, String> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name).cloned() {
                return Ok(value);
            }
        }
        Err(utils::get_error_message("RUNTIME", &[("variable", name)])?)
    }

    fn change_variable(&mut self, name: String, value: GreenType) -> Result<(), String> {
        if let Some(scope) = self.scopes.last_mut() {
            if let Some(variable) = scope.get_mut(&name) {
                *variable = value;
                return Ok(());
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
    functions: HashMap<String, (Vec<String>, Node)>,
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
        match &node.kind {
            NodeKind::Program => {
                for child in &node.children {
                    self.statement(child)?;
                }
            },
            _ => return Err(utils::get_error_message("RUNTIME003", &[])?),
        }
        Ok(())
    }

    fn statement(&mut self, node: &Node) -> Result<(), String> {
        match &node.kind {
            NodeKind::FunctionCall { name: _, arguments: _ } => self.execute_function(node)?,
            NodeKind::VariableDeclaration { name } => {
                let expression = self.evaluate_assignable(&node.children[0])?;
                self.variables.set_variable(name.to_string(), expression);
            },
            NodeKind::VariableAssignment { name } => {
                let expression = self.evaluate_assignable(&node.children[0])?;
                self.variables.change_variable(name.to_string(), expression)?;
            },
            NodeKind::IfStatement => self.evaluate_if_statement(node)?,
            NodeKind::FunctionDefinition { name, parameters } => {
                let program = &node.children[0];
                self.functions.insert(name.to_string(), (parameters.clone(), program.clone()));
            },
            _ => return Err(utils::get_error_message("RUNTIME003", &[])?),
        }
        Ok(())
    }

    /// print関数の実行
    fn print_function(&mut self, arguments: &Vec<Node>) -> Result<(), String> {
        let values = self.evaluate_argument(arguments)?;
        let result = values.iter().map(ToString::to_string).collect::<Vec<_>>().join(" ");
        println!("{}", result);
        Ok(())
    }

    fn execute_function(&mut self, node: &Node) -> Result<(), String> {
        match &node.kind {
            NodeKind::FunctionCall { name, arguments } => {
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
                            for (param, value) in parameters.iter().zip(values.iter()) {
                                self.variables.set_variable(param.clone(), value.clone());
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

    fn evaluate_if_statement(&mut self, node: &Node) -> Result<(), String> {
        let condition_node = &node.children[0];
        if let GreenType::Bool(condition_result) = self.evaluate_assignable(&condition_node)? {
            if condition_result {
                let then_block = &node.children[1];
                self.execute(then_block)?;
            } else {
                let else_block = &node.children[2];
                self.execute(else_block)?;
            }
        } else {
            return Err(format!("if文の条件が正しくない"))
        }
        Ok(())
    }

    /// 引数の評価
    fn evaluate_argument(&mut self, arguments: &Vec<Node>) -> Result<Vec<GreenType>, String> {
        let mut values = Vec::new();
        for child in arguments {
            values.push(self.evaluate_assignable(child)?);
        }
        return Ok(values)
    }

    /// 割り当て可能値の評価（引数、代入式の右辺）
    fn evaluate_assignable(&mut self, node: &Node) -> Result<GreenType, String> {
        match &node.kind {
            NodeKind::Compare{ operator: _, left: _, right: _ } | NodeKind::Arithmetic{ operator: _, left: _, right: _ }
            | NodeKind::Variable { name: _ } | NodeKind::Logical{ operator: _, left: _, right: _ } => {
                self.evaluate_expression(node)
            },
            NodeKind::Literal{ value: _ } => self.evaluate_literal(node),
            _ => Err(utils::get_error_message("RUNTIME005", &[])?),
        }
    }

    /// 式の評価
    fn evaluate_expression(&mut self, node: &Node) -> Result<GreenType, String> {
        match &node.kind {
            NodeKind::Logical {
                operator,
                left,
                right,
            } => {
                match operator {
                    Logical::Unary(unary_operator) => {
                        let node = left;
                        if let GreenType::Bool(value)  = self.evaluate_expression(node)? {
                            let result = self.unary_logical_operations(unary_operator, value)?;
                            Ok(GreenType::Bool(result))
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
                            (GreenType::Bool(left_value), GreenType::Bool(right_value)) => {
                                let result = self.binary_logical_operations(binary_operator, left_value, right_value)?;
                                Ok(GreenType::Bool(result))
                            },
                            (left_value, right_value) => {
                                Err(format!("想定外の論理演算: 左: {:?} 演算子: {:?} 右: {:?}", left_value, binary_operator, right_value))
                            },
                        }
                    },
                }
            },
            NodeKind::Compare {
                operator,
                left,
                right
            } => {
                let left = self.evaluate_expression(left)?;
                let right = self.evaluate_expression(right)?;
                match (left, right) {
                    (GreenType::Float(left_value), GreenType::Float(right_value)) => {
                        let result = self.compare_values(operator, left_value, right_value)?;
                        Ok(GreenType::Bool(result))
                    },
                    (GreenType::String(left_value), GreenType::String(right_value)) => {
                        match operator {
                            Comparison::Equal => Ok(GreenType::Bool(left_value == right_value)),
                            Comparison::NotEqual => Ok(GreenType::Bool(left_value != right_value)),
                            _ => Err(utils::get_error_message("RUNTIME006", &[("operator", format!("{:?}", operator).as_str())])?),
                        }
                    },
                    (left_value, right_value) => {
                        Err(format!("異なる型の比較はできません: 左: {:?} 右: {:?}", left_value, right_value))
                    },
                }
            },
            // 四則演算
            NodeKind::Arithmetic {
                operator: Arithmetic::Binary(binary_operator),
                left,
                right
            } => {
                match self.evaluate_expression(left)? {
                    GreenType::Float(left_value) => {
                        let right = match right {
                            Some(right) => right,
                            None => return Err(utils::get_error_message("RUNTIME009", &[])?),
                        };
                        let right_value = if let GreenType::Float(right_value) = self.evaluate_expression(right)?{
                            right_value
                        } else {
                            return Err(format!("無効な演算: {:?}", node));
                        };
                        match binary_operator {
                            BinaryArithmetic::Add => Ok(GreenType::Float(left_value + right_value)),
                            BinaryArithmetic::Subtract => Ok(GreenType::Float(left_value - right_value)),
                            BinaryArithmetic::Multiply => Ok(GreenType::Float(left_value * right_value)),
                            BinaryArithmetic::Divide => Ok(GreenType::Float(left_value / right_value)),
                        }
                    },
                    _ => Err(format!("想定外のAddAndSub型: {:?}", node))
                }
            },
            NodeKind::Arithmetic {
                operator: Arithmetic::Unary(unary_operator),
                left,
                right: _,
            } => {
                if let GreenType::Float(value) = self.evaluate_expression(left)?{
                    let mut result = value;
                    if *unary_operator == UnaryArithmetic::Minus {
                        result = -1.0 * value;
                    }
                    Ok(GreenType::Float(result))
                } else {
                    Err(format!("想定外のPrimary型: {:?}", left))
                }
            },
            NodeKind::Variable { name } => {
                let variable = self.variables.get_variable(name)?;
                Ok(variable)
            },
            NodeKind::Literal { value: _ } => self.evaluate_literal(node),
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
    fn evaluate_literal(&mut self, node: &Node) -> Result<GreenType, String> {
        match &node.kind {
            NodeKind::Literal { value: LiteralValue::String(string) } => Ok(GreenType::String(string.to_string())),
            NodeKind::Literal { value: LiteralValue::Float(float) } => Ok(GreenType::Float(*float)),
            NodeKind::Literal { value: LiteralValue::Bool(bool ) } => Ok(GreenType::Bool(*bool)),
            _ => { Err(format!("想定外のリテラルの型: {:?}", node)) }
        }
    }

}

pub fn execute(node: &Node) -> Result<(), String> {
    let mut interpreter = Interpreter::new();
    interpreter.execute(node)?;
    Ok(())
}