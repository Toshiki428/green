use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;
use crate::parser::node::Node;
use std::fs::File;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonData {
    definitions: Vec<Definition>,
    structures: Structures,
}
impl JsonData {
    pub fn new(ast: Node) {
        let mut json_data = Self {
            definitions: Vec::new(),
            structures: Structures::new(),
        };

        let _ = json_data.ast_to_json(ast);
    }

    pub fn ast_to_json(&mut self, ast: Node) -> serde_json::Result<()> {    
        match ast {
            Node::Program { functions, coroutines } => {
                for function in functions {
                    match function {
                        Node::FunctionDefinition { name, parameters:_, block , doc} => {
                            self.definitions.push(Definition::new(&name, "function", &doc));
                            let stack = AnalyzeAst::new(*block);
                            self.structures.insert(name, stack);
                        },
                        _ => {},
                    }
                }
    
                for coroutine in coroutines {
                    match coroutine {
                        Node::CoroutineDefinition { name, block, doc } => {
                            self.definitions.push(Definition::new(&name, "coroutine", &doc));
                            let stack = AnalyzeAst::new(*block);
                            self.structures.insert(name, stack);
                        },
                        _ => {},
                    }
                }
            },
            _ => {},
        }
    
        // シリアライズ
        let serialized = serde_json::to_string_pretty(&self)?;
    
        let mut file = File::create("analyze.json").expect("Unable to create file");
        writeln!(file, "{}", serialized).expect("Unable to write data");
    
        Ok(())
    }
    
}

#[derive(Serialize, Deserialize, Debug)]
struct Definition {
    name: String,
    r#type: String,
    doc: String,
}
impl Definition {
    fn new(name: &str, r#type: &str, doc: &Option<String>) -> Self {
        let doc = match doc {
            Some(doc) => doc,
            None => "",
        };
        Self { name: name.to_string(), r#type: r#type.to_string(), doc: doc.to_string() }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    r#type: String,
    data: Value,
}
impl Data {
    fn new(r#type: &str, data: Value) -> Self {
        Self {
            r#type: r#type.to_string(),
            data,
        }
    }
}

type Structures = HashMap<String, Vec<Data>>;

struct AnalyzeAst {
    stack: Vec<Data>,
}
impl AnalyzeAst {
    fn new(ast: Node) -> Vec<Data> {
        let mut analyze_ast = Self{
            stack: Vec::new(),
        };
        analyze_ast.analyze_ast(ast);
        return analyze_ast.stack;
    }

    fn analyze_ast(&mut self, ast: Node) {
        match ast {
            Node::Block { block_type:_, statements } => {
                for statement in statements {
                    self.analyze_ast(statement)
                }
            },

            Node::IfStatement { condition_node:_, then_block, else_block } => {
                self.analyze_ast(*then_block);
                if let Some(else_block) = else_block {
                    self.analyze_ast(*else_block);
                }
            },
            Node::LoopStatement { condition_node:_, block } => {
                self.analyze_ast(*block);
            },

            Node::VariableDeclaration { name:_, variable_type:_, initializer, doc:_ } => {
                if let Some(ini) = initializer {
                    self.analyze_ast(*ini);
                }
            },
            Node::VariableAssignment { name:_, expression } => {
                self.analyze_ast(*expression);
            },

            Node::CoroutineInstantiation { task_name:_, coroutine_name:_ } => {
                // あとで
            },
            Node::CoroutineResume { task_name:_ } => {
                // あとで
            },

            Node::FunctionCall { name, arguments } | Node::FunctionCallWithReturn { name, arguments } => {
                if name != "print" {
                    self.stack.push(Data::new(
                        "function_call",
                        serde_json::json!({
                            "target": &name
                        }),
                    ));
                }

                for arg in arguments {
                    self.analyze_ast(arg);
                }
            },
            
            Node::ProcessComment { comment } => {
                self.stack.push(Data::new(
                    "doc",
                    serde_json::json!({
                        "string": &comment
                    }),
                ));
            },

            _ => {},
        }
    }
}
