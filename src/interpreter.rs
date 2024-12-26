use std::collections::HashMap;

use crate::{parser::{LiteralValue, Node, NodeKind}, utils};

#[derive(Debug, Clone)]
enum GreenType {
    Float(f64),
    // Int(i32),
    Bool(bool),
    String(String),
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

    // fn push_scope(&mut self) {
    //     self.scopes.push(HashMap::new());
    // }

    // fn pop_scope(&mut self) {
    //     self.scopes.pop();
    // }
}

struct Interpreter {
    variables: Environment,
}

impl Interpreter {
    fn new() -> Self {
        Interpreter {
            variables: Environment::new(),
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
            NodeKind::FunctionCall { name } => {
                match name.as_str() {
                    "print" => { self.print_function(node)?; },
                    _ => { return Err(utils::get_error_message("RUNTIME002", &[("function", name)])?) },
                }
            },
            NodeKind::VariableDeclaration { name } => {
                let expression = self.evaluate_assignable(node)?;
                self.variables.set_variable(name.to_string(), expression);
            },
            _ => return Err(utils::get_error_message("RUNTIME003", &[])?),
        }
        Ok(())
    }

    /// print関数の実行
    fn print_function(&mut self, node: &Node) -> Result<(), String> {
        let argument = node.children.get(0).ok_or(utils::get_error_message("RUNTIME004", &[])?)?;
        if argument.kind != NodeKind::Argument {
            return Err(utils::get_error_message("RUNTIME005", &[])?);
        }

        let value = self.evaluate_argument(argument)?;
        match value {
            GreenType::Float(value) => println!("{}", value),
            // GreenType::Int(value) => println!("{}", value),
            GreenType::Bool(value) => println!("{}", value),
            GreenType::String(value) => println!("{}", value),
        }
        Ok(())
    }

    /// 引数の評価
    fn evaluate_argument(&mut self, node: &Node) -> Result<GreenType, String> {
        return self.evaluate_assignable(node)
    }

    /// 割り当て可能値の評価（引数、代入式の右辺）
    fn evaluate_assignable(&mut self, node: &Node) -> Result<GreenType, String> {
        let child = node.children.get(0).ok_or(utils::get_error_message("RUNTIME004", &[])?)?;
        match &child.kind {
            NodeKind::Compare { operator: _ } | NodeKind::AddAndSub { operator: _ } 
            | NodeKind::MulAndDiv { operator: _ } | NodeKind::Unary { operator: _ }
            | NodeKind::Variable { name: _ } => {
                self.evaluate_expression(child)
            },
            NodeKind::Literal(_) => self.evaluate_literal(child),
            _ => Err(utils::get_error_message("RUNTIME005", &[])?),
        }
    }

    /// 式の評価
    fn evaluate_expression(&mut self, node: &Node) -> Result<GreenType, String> {
        match &node.kind {
            NodeKind::Compare { operator } => {
                let left_node = node.children.get(0).ok_or(utils::get_error_message("RUNTIME004", &[])?)?;
                let left = self.evaluate_expression(left_node)?;
                
                let right_node = node.children.get(1).ok_or(utils::get_error_message("RUNTIME004", &[])?)?;
                let right = self.evaluate_expression(right_node)?;

                match (left, right) {
                    (GreenType::Float(left_value), GreenType::Float(right_value)) => {
                        let result = self.compare_values(operator, left_value, right_value)?;
                        Ok(GreenType::Bool(result))
                    },
                    (GreenType::String(left_value), GreenType::String(right_value)) => {
                        match operator.as_str() {
                            "==" => Ok(GreenType::Bool(left_value == right_value)),
                            "!=" => Ok(GreenType::Bool(left_value != right_value)),
                            _ => Err(utils::get_error_message("RUNTIME006", &[("operator", operator)])?),
                        }
                    },
                    (left_value, right_value) => {
                        Err(format!("異なる型の比較はできません: 左: {:?} 右: {:?}", left_value, right_value))
                    },
                }
            },
            // 四則演算
            NodeKind::AddAndSub { operator } | NodeKind::MulAndDiv { operator } => {
                let left = node.children.get(0).ok_or(utils::get_error_message("RUNTIME004", &[])?)?;
                match self.evaluate_expression(left)? {
                    GreenType::Float(left_value) => {
                        let right = node.children.get(1).ok_or(utils::get_error_message("RUNTIME004", &[])?)?;
                        let right_value = if let GreenType::Float(right_value) = self.evaluate_expression(right)?{
                            right_value
                        } else {
                            return Err(format!("無効な演算: {:?}", node));
                        };
                        match operator.as_str() {
                            "+" => Ok(GreenType::Float(left_value + right_value)),
                            "-" => Ok(GreenType::Float(left_value - right_value)),
                            "*" => Ok(GreenType::Float(left_value * right_value)),
                            "/" => Ok(GreenType::Float(left_value / right_value)),
                            _ => Err(format!("想定外の演算子: {}", operator)),
                        }
                    },
                    _ => Err(format!("想定外のAddAndSub型: {:?}", node))
                }
            },
            NodeKind::Unary { operator } => {
                let number = node.children.get(0).ok_or(utils::get_error_message("RUNTIME004", &[])?)?;
                if let GreenType::Float(value) = self.evaluate_expression(number)?{
                    let mut result = value;
                    if operator == "-" {
                        result = -1.0 * value;
                    }
                    Ok(GreenType::Float(result))
                } else {
                    Err(format!("想定外のPrimary型: {:?}", number))
                }
            },
            NodeKind::Variable { name } => {
                let variable = self.variables.get_variable(name)?;
                Ok(variable)
            },
            NodeKind::Literal(_) => self.evaluate_literal(node),
            _ => Err(utils::get_error_message("RUNTIME003", &[])?),
        }
    }

    /// 比較処理
    fn compare_values(&mut self, operator: &str, left: f64, right: f64) -> Result<bool, String> {
        match operator {
            "==" => Ok(left == right),
            "!=" => Ok(left != right),
            ">=" => Ok(left >= right),
            ">" => Ok(left > right),
            "<=" => Ok(left <= right),
            "<" => Ok(left < right),
            _ => Err(utils::get_error_message("RUNTIME008", &[])?),
        }
    }

    /// 値の評価
    fn evaluate_literal(&mut self, node: &Node) -> Result<GreenType, String> {
        match &node.kind {
            NodeKind::Literal(LiteralValue::String(value)) => Ok(GreenType::String(value.to_string())),
            NodeKind::Literal(LiteralValue::Float(value)) => Ok(GreenType::Float(*value)),
            NodeKind::Literal(LiteralValue::Bool(value)) => Ok(GreenType::Bool(*value)),
            _ => { Err(format!("想定外のリテラルの型: {:?}", node)) }
        }
    }

}

pub fn execute(node: &Node) -> Result<(), String> {
    let mut interpreter = Interpreter::new();
    interpreter.execute(node)?;
    Ok(())
}