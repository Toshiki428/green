use crate::parser::{Node, NodeKind};

#[derive(Debug)]
enum EvaluateExpressionType {
    Number(f64),
    Bool(bool),
}

/// プログラムの実行
/// 
/// ## Argments
/// 
/// - `node` - ASTのノード
/// 
/// ## Return
/// 
/// - 実行結果
/// 
/// ## Example
/// 
/// ```
/// if let Err(e) = interpreter::execute_ast(&ast) { 
///     eprintln!("Error execute: {}", e);
///     return;
/// }
/// ```
pub fn execute_ast(node: &Node) -> Result<(), String> {
    match &node.kind {
        NodeKind::Program => {
            for child in &node.children {
                execute_ast(child)?;
            }
            Ok(())
        },
        NodeKind::FunctionCall { name } => {
            if name == "print" {
                print_function(node)
            } else {
                Err(format!("Unknown function: {}", name))
            }
        },
        _ => Err("Unsupported node type".to_string()),
    }
}

fn print_function(node: &Node) -> Result<(), String> {
    if let Some(argument) = node.children.get(0) {
        match &argument.kind {
            NodeKind::Argument => {
                if let Some(first_child) = argument.children.get(0){
                    match &first_child.kind {
                        NodeKind::String { value } => {
                            println!("{}", value);
                            Ok(())
                        },
                        NodeKind::Expression => {
                            if let Some(expression_child) = first_child.children.get(0){
                                if let Some(result) = evaluate_expression(expression_child) {
                                    println!("{:?}", result);
                                    Ok(())
                                } else {
                                    Err("Failed to evaluate the numerical expression".to_string())
                                }
                            } else {
                                Err("Err".to_string())
                            }
                        },
                        NodeKind::Bool { value } => {
                            println!("{}", value);
                            Ok(())
                        },
                        _ => Err("Unsupported argument type in Argument node".to_string()),
                    }
                } else {
                    Err("Argument node is empty".to_string())
                }
            },
            _ => Err("Invalid argument to function 'print'".to_string()),
        }
    } else {
        Err("Missing argument to function 'print'".to_string())
    }
}

fn evaluate_expression(node: &Node) -> Option<EvaluateExpressionType> {
    match &node.kind {
        NodeKind::Float { value } => Some( EvaluateExpressionType::Number(*value) ),
        NodeKind::Int { value } => Some( EvaluateExpressionType::Number( *value as f64 ) ),
        NodeKind::Number => {
            let number = node.children.get(0)?;
            let value = evaluate_expression(number)?;
            Some(value)
        },
        NodeKind::Primary => {
            let number = node.children.get(0)?;
            let value = evaluate_expression(number)?;
            Some(value)
        },
        NodeKind::MulAndDiv { operator } => {
            let left = node.children.get(0)?;
            match evaluate_expression(left) {
                Some(EvaluateExpressionType::Number(left_value)) => {
                    match operator.as_str() {
                        "*" => {
                            let right = node.children.get(1)?;
                            if let EvaluateExpressionType::Number(right_value) = evaluate_expression(right)?{
                                return Some(EvaluateExpressionType::Number(left_value * right_value));
                            } else {
                                return None;
                            }
                        },
                        "/" => {
                            let right = node.children.get(1)?;
                            if let EvaluateExpressionType::Number(right_value) = evaluate_expression(right)?{
                                return Some(EvaluateExpressionType::Number(left_value / right_value));
                            } else {
                                return None;
                            }
                        },
                        "" => Some(EvaluateExpressionType::Number(left_value)),
                        _ => None,
                    }
                },
                Some(EvaluateExpressionType::Bool(value)) => Some(EvaluateExpressionType::Bool(value)),
                _ => None,
            }
        },
        NodeKind::AddAndSub { operator } => {
            let left = node.children.get(0)?;
            match evaluate_expression(left) {
                Some(EvaluateExpressionType::Number(left_value)) => {
                    match operator.as_str() {
                        "+" => {
                            let right = node.children.get(1)?;
                            if let EvaluateExpressionType::Number(right_value) = evaluate_expression(right)?{
                                return Some(EvaluateExpressionType::Number(left_value + right_value));
                            } else {
                                return None;
                            }
                        },
                        "-" => {
                            let right = node.children.get(1)?;
                            if let EvaluateExpressionType::Number(right_value) = evaluate_expression(right)?{
                                return Some(EvaluateExpressionType::Number(left_value - right_value));
                            } else {
                                return None;
                            }
                        },
                        "" => Some(EvaluateExpressionType::Number(left_value)),
                        _ => None,
                    }
                },
                Some(EvaluateExpressionType::Bool(value)) => { return Some(EvaluateExpressionType::Bool(value)); }
                _ => { return None; }
            }
        }
        NodeKind::Compare { operator } => {
            let left = node.children.get(0)?;
            match evaluate_expression(left) {
                Some(EvaluateExpressionType::Number(left_value)) => {
                    match operator.as_str() {
                        "==" => {
                            let right = node.children.get(1)?;
                            if let EvaluateExpressionType::Number(right_value) = evaluate_expression(right)?{
                                return Some(EvaluateExpressionType::Bool(left_value == right_value));
                            } else {
                                return None;
                            }
                        },
                        "!=" => {
                            let right = node.children.get(1)?;
                            if let EvaluateExpressionType::Number(right_value) = evaluate_expression(right)?{
                                return Some(EvaluateExpressionType::Bool(left_value != right_value));
                            } else {
                                return None;
                            }
                        },
                        "" => Some(EvaluateExpressionType::Number(left_value)),
                        _ => None,
                    }
                }
                _ => None,
            }
        },
        _ => None,
    }
}