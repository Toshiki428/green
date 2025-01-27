use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::parser::node::{RootNode, GlobalNode, PrivateNode};
use std::{
    fs::File,
    io::Write,
    collections::HashMap,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonData {
    definitions: Vec<Definition>,
    structures: HashMap<String, Vec<Data>>,
}
impl JsonData {
    pub fn new(ast: RootNode) {
        let mut json_data = Self {
            definitions: Vec::new(),
            structures: HashMap::new(),
        };

        let _ = json_data.ast_to_json(ast);
    }

    pub fn ast_to_json(&mut self, ast: RootNode) -> serde_json::Result<()> {    
        match ast {
            RootNode::Program { functions, coroutines } => {
                for function in functions {
                    match function {
                        GlobalNode::FunctionDefinition { name, parameters:_, block , doc} => {
                            self.definitions.push(Definition::new(&name, "function", &doc));
                            let stack = AnalyzeAst::new(block);
                            self.structures.insert(name, stack);
                        },
                        _ => {},
                    }
                }
    
                for coroutine in coroutines {
                    match coroutine {
                        GlobalNode::CoroutineDefinition { name, block, doc } => {
                            self.definitions.push(Definition::new(&name, "coroutine", &doc));
                            let stack = AnalyzeAst::new(block);
                            self.structures.insert(name, stack);
                        },
                        _ => {},
                    }
                }
            },
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

struct AnalyzeAst {
    stack: Vec<Data>,
}
impl AnalyzeAst {
    fn new(ast: PrivateNode) -> Vec<Data> {
        let mut analyze_ast = Self{
            stack: Vec::new(),
        };
        analyze_ast.analyze_ast(ast);
        return analyze_ast.stack;
    }

    fn analyze_ast(&mut self, ast: PrivateNode) {
        match ast {
            PrivateNode::Block { block_type:_, statements } => {
                for statement in statements {
                    self.analyze_ast(statement)
                }
            },

            PrivateNode::IfStatement { condition_node:_, then_block, else_block } => {
                self.analyze_ast(*then_block);
                if let Some(else_block) = else_block {
                    self.analyze_ast(*else_block);
                }
            },
            PrivateNode::LoopStatement { condition_node:_, block } => {
                self.analyze_ast(*block);
            },

            PrivateNode::VariableDeclaration { name:_, variable_type:_, initializer, doc:_ } => {
                if let Some(ini) = initializer {
                    self.analyze_ast(*ini);
                }
            },
            PrivateNode::VariableAssignment { name:_, expression } => {
                self.analyze_ast(*expression);
            },

            PrivateNode::CoroutineInstantiation { task_name:_, coroutine_name:_ } => {
                // あとで
            },
            PrivateNode::CoroutineResume { task_name:_ } => {
                // あとで
            },

            PrivateNode::FunctionCall { name, arguments } | PrivateNode::FunctionCallWithReturn { name, arguments } => {
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
            
            PrivateNode::ProcessComment { comment } => {
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
