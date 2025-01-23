use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;
use crate::parser::node::Node;

#[derive(Serialize, Deserialize, Debug)]
struct JsonData {
    definitions: Vec<Definition>,
    structures: Structures,
}

#[derive(Serialize, Deserialize, Debug)]
struct Definition {
    name: String,
    r#type: String,
    doc: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    r#type: String,
    data: Value,
}

type Structures = HashMap<String, Vec<Data>>;

pub fn ast_to_json(ast: Node) -> serde_json::Result<()> {
    let mut definitions = Vec::new();
    let mut structures = HashMap::new();

    match ast {
        Node::Block { block_type:_, statements } => {
            for statement in statements {
                match statement {
                    Node::FunctionDefinition { name, parameters:_, block:_ } => {
                        definitions.push(Definition {
                            name,
                            r#type: "function".to_string(),
                            doc: "".to_string(),
                        });
                    },
                    Node::FunctionCall { name, arguments:_ } => {
                        structures.insert(
                            name,
                            vec![
                                Data {
                                    r#type: String::from("doc"),
                                    data: serde_json::json!({
                                        "string": "..."
                                    }),
                                },
                                Data {
                                    r#type: String::from("function_call"),
                                    data: serde_json::json!({
                                        "target": "main::file_load"
                                    }),
                                },
                            ],
                        );
                    },
                    _ => {},
                }
            }
        },
        _ => {},
    }

    let json_data = JsonData {definitions, structures};

    // シリアライズ
    let serialized = serde_json::to_string_pretty(&json_data)?;
    println!("Serialized JSON:\n{}", serialized);

    // デシリアライズ
    let deserialized: JsonData = serde_json::from_str(&serialized)?;
    println!("\nDeserialized Structures:\n{:#?}", deserialized);

    Ok(())
}