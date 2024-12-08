use crate::parser::{Node, NodeKind};

#[derive(Debug)]
enum ArgumentType {
    Number(f64),
    Bool(bool),
    String(String),
}

/// プログラムの実行
/// 
/// ## Arguments
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
/// if let Err(e) = interpreter::execute(&ast) {
///     eprintln!("実行エラー: {}", e);
///     return;
/// }
/// ```
pub fn execute(node: &Node) -> Result<(), String> {
    match &node.kind {
        NodeKind::Program => {
            for child in &node.children {
                execute(child)?;
            }
        },
        NodeKind::FunctionCall { name } => {
            match name.as_str() {
                "print" => { print_function(node)?; },
                _ => { return Err(format!("未定義の関数: {}", name)) },
            }
        },
        _ => return Err("想定外のNodeKind".to_string()),
    }
    Ok(())
}

/// print関数の実行
/// 
/// ## Arguments
/// 
/// - `node`
/// 
/// ## Return
/// 
/// - 実行結果
/// 
/// ## Example
/// 
/// ```
/// print_function(node)?;
/// ```
fn print_function(node: &Node) -> Result<(), String> {
    let argument = node.children.get(0).ok_or("'print'関数の引数がない")?;
    if argument.kind != NodeKind::Argument {
        return Err("想定外の'print'関数の引数".to_string());
    }

    let value = evaluate_argument(argument)?;
    match value {
        ArgumentType::Number(number) => println!("{}", number),
        ArgumentType::Bool(value) => println!("{}", value),
        ArgumentType::String(value) => println!("{}", value),
    }
    Ok(())
}

/// 引数の評価
/// ## Arguments
/// 
/// - `node`
/// 
/// ## Return
/// 
/// - 評価結果
/// 
/// ## Example
/// 
/// ```
/// let result = evaluate_argument(node)?;
/// ```
fn evaluate_argument(node: &Node) -> Result<ArgumentType, String> {
    let first_child = node.children.get(0).ok_or(format!("引数が空: {:?}", node))?;
    match &first_child.kind {
        NodeKind::Expression => {
            let expression_node = first_child.children.get(0).ok_or(format!("計算式が空: {:?}", first_child))?;
            let expression = evaluate_expression(expression_node)?;
            return Ok(expression);
        },
        NodeKind::Bool { value } => return Ok(ArgumentType::Bool(*value)),
        _ => return Err("想定外の引数の型".to_string()),
    }
}

/// 式の評価
/// 
/// ## Arguments
/// 
/// - `node`
/// 
/// ## Return
/// 
/// - 評価結果
/// 
/// ## Example
/// 
/// ```
/// let result = evaluate_expression(node)?;
/// ```
fn evaluate_expression(node: &Node) -> Result<ArgumentType, String> {
    match &node.kind {
        NodeKind::Compare { operator } => {
            let left_node = node.children.get(0).ok_or(format!("式が無効: {:?}", node))?;
            let left = evaluate_value(left_node)?;
            match left {
                ArgumentType::Number(left_value) => {
                    match operator.as_str() {
                        "==" => {
                            let right_node = node.children.get(1).ok_or(format!("式が無効: {:?}", node))?;
                            if let ArgumentType::Number(right_value) = evaluate_value(right_node)?{
                                return Ok(ArgumentType::Bool(left_value == right_value));
                            } else {
                                return Err(format!("Number型とString型の比較はできません: {:?}", node));
                            }
                        },
                        "!=" => {
                            let right_node = node.children.get(1).ok_or(format!("式が無効: {:?}", node))?;
                            if let ArgumentType::Number(right_value) = evaluate_value(right_node)?{
                                return Ok(ArgumentType::Bool(left_value != right_value));
                            } else {
                                return Err(format!("Number型とString型の比較はできません: {:?}", node));
                            }
                        },
                        "" => Ok(ArgumentType::Number(left_value)),
                        _ => Err(format!("想定外の比較演算子: {}", operator)),
                    }
                },
                ArgumentType::String(left_value) => {
                    match operator.as_str() {
                        "==" => {
                            let right_node = node.children.get(1).ok_or(format!("式が無効: {:?}", node))?;
                            if let ArgumentType::String(right_value) = evaluate_value(right_node)?{
                                return Ok(ArgumentType::Bool(left_value == right_value));
                            } else {
                                return Err(format!("String型とNumber型の比較はできません: {:?}", node));
                            }
                        },
                        "!=" => {
                            let right_node = node.children.get(1).ok_or(format!("式が無効: {:?}", node))?;
                            if let ArgumentType::String(right_value) = evaluate_value(right_node)?{
                                return Ok(ArgumentType::Bool(left_value != right_value));
                            } else {
                                return Err(format!("String型とNumber型の比較はできません: {:?}", node));
                            }
                        },
                        "" => Ok(ArgumentType::String(left_value)),
                        _ => Err(format!("想定外の比較演算子: {}", operator)),
                    }
                },
                _ => Err(format!("想定外の式の型: {:?}", left)),
            }
        },        
        _ => Err(format!("想定外の型: {:?}", node)),
    }
}

/// 値の評価
/// 
/// ## Arguments
/// 
/// - `node`
/// 
/// ## Return
/// 
/// - 評価結果
/// 
/// ## Example
/// 
/// ```
/// let result = evaluate_value(node)?;
/// ```
fn evaluate_value(node: &Node) -> Result<ArgumentType, String> {
    match &node.kind {
        NodeKind::String { value } => Ok(ArgumentType::String(value.to_string())),
        NodeKind::AddAndSub { operator } => {
            let left = node.children.get(0).ok_or(format!("式が無効: {:?}", node))?;
            match evaluate_value(left)? {
                ArgumentType::Number(left_value) => {
                    match operator.as_str() {
                        "+" => {
                            let right = node.children.get(1).ok_or(format!("式が無効: {:?}", node))?;
                            if let ArgumentType::Number(right_value) = evaluate_value(right)?{
                                return Ok(ArgumentType::Number(left_value + right_value));
                            } else {
                                return Err(format!("無効な足し算: {:?}", node));
                            }
                        },
                        "-" => {
                            let right = node.children.get(1).ok_or(format!("式が無効: {:?}", node))?;
                            if let ArgumentType::Number(right_value) = evaluate_value(right)?{
                                return Ok(ArgumentType::Number(left_value - right_value));
                            } else {
                                return Err(format!("無効な引き算: {:?}", node));
                            }
                        },
                        "" => Ok(ArgumentType::Number(left_value)),
                        _ => Err(format!("想定外の演算子: {}", operator)),
                    }
                },
                _ => Err(format!("想定外のAddAndSub型: {:?}", node))
            }
        },
        NodeKind::MulAndDiv { operator } => {
            let left = node.children.get(0).ok_or(format!("式が無効: {:?}", node))?;
            match evaluate_value(left)? {
                ArgumentType::Number(left_value) => {
                    match operator.as_str() {
                        "*" => {
                            let right = node.children.get(1).ok_or(format!("式が無効: {:?}", node))?;
                            if let ArgumentType::Number(right_value) = evaluate_value(right)?{
                                return Ok(ArgumentType::Number(left_value * right_value));
                            } else {
                                return Err(format!("無効な掛け算: {:?}", node));
                            }
                        },
                        "/" => {
                            let right = node.children.get(1).ok_or(format!("式が無効: {:?}", node))?;
                            if let ArgumentType::Number(right_value) = evaluate_value(right)?{
                                if right_value == 0.0 {
                                    return Err(format!("0で割ることはできません: {:?}", node));
                                }
                                return Ok(ArgumentType::Number(left_value / right_value));
                            } else {
                                return Err(format!("無効な割り算: {:?}", node));
                            }
                        },
                        "" => Ok(ArgumentType::Number(left_value)),
                        _ => Err(format!("想定外の演算子: {}", operator)),
                    }
                },
                _ => Err(format!("想定外のMulAndDiv型: {:?}", node)),
            }
        },
        NodeKind::Unary { operator } => {
            let number = node.children.get(0).ok_or(format!("式が無効: {:?}", node))?;
            if let ArgumentType::Number(value) = evaluate_value(number)?{
                let mut result = value;
                if operator == "-" {
                    result = -1.0 * value;
                }
                Ok(ArgumentType::Number(result))
            } else {
                Err(format!("想定外のPrimary型: {:?}", number))
            }
        },
        NodeKind::Primary => {
            let number = node.children.get(0).ok_or(format!("式が無効: {:?}", node))?;
            let value = evaluate_value(number)?;
            Ok(value)
        },
        NodeKind::Number { value } => Ok(ArgumentType::Number(*value)),
        _ => { Err(format!("想定外のリテラルの型: {:?}", node)) }
    }
}
