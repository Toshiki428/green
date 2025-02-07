use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::{analyzer::semantic::Semantic, parser::node::*};
use std::{
    fs::File,
    io::Write,
    collections::BTreeMap,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonData {
    definitions: Vec<Definition>,
    structures: BTreeMap<String, Vec<Data>>,
}
impl JsonData {
    pub fn new(semantic: Semantic) {
        let mut json_data = Self {
            definitions: Vec::new(),
            structures: BTreeMap::new(),
        };

        let _ = json_data.ast_to_json(semantic);
    }

    pub fn ast_to_json(&mut self, semantic: Semantic) -> serde_json::Result<()> {
        for (_, function_info) in semantic.function_table.table {
            if function_info.name == "print" {
                continue;
            }

            self.definitions.push(Definition::new(&function_info.name, "function", &function_info.doc, ""));
            let stack = AnalyzeAst::new(function_info.process);
            self.structures.insert(function_info.name, stack);
        }

        for (_, task) in semantic.task_table.table {
            let coroutine = semantic.coroutine_table.get_coroutine_info(&task.coroutine_name).unwrap();
            self.definitions.push(Definition::new(&task.task_name, "coroutine", &coroutine.doc, &coroutine.name));
            let stack = AnalyzeAst::new(coroutine.process);
            self.structures.insert(task.task_name, stack);
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
    r#ref: String,
}
impl Definition {
    fn new(name: &str, r#type: &str, doc: &str, r#ref: &str) -> Self {
        Self { name: name.to_string(), r#type: r#type.to_string(), doc: doc.to_string(), r#ref: r#ref.to_string() }
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
    fn new(ast: BlockNode) -> Vec<Data> {
        let mut analyze_ast = Self{
            stack: Vec::new(),
        };
        analyze_ast.analyze_block(ast);
        return analyze_ast.stack;
    }

    fn analyze_block(&mut self, block: BlockNode) {
        for statement in block.statements {
            self.analyze_node(statement);
        }
    }

    fn analyze_node(&mut self, ast: PrivateNode) {
        match ast {
            PrivateNode::IfStatement { condition_node:_, then_block, else_block } => {
                self.analyze_block(then_block);
                if let Some(else_block) = else_block {
                    self.analyze_block(else_block);
                }
            },
            PrivateNode::LoopStatement { condition_node:_, block } => {
                self.analyze_block(block);
            },

            PrivateNode::VariableDeclaration { name:_, variable_type:_, initializer, doc:_ } => {
                if let Some(ini) = initializer {
                    self.analyze_node(*ini);
                }
            },
            PrivateNode::VariableAssignment { name:_, expression } => {
                self.analyze_node(*expression);
            },

            PrivateNode::CoroutineInstantiation { task_name:_, coroutine_name:_ } => {},
            PrivateNode::CoroutineResume { task_name } => {
                self.stack.push(Data::new(
                    "task_resume",
                    serde_json::json!({
                        "target": &task_name,
                    }),
                ));
            },
            PrivateNode::Yield => {
                self.stack.push(Data::new(
                    "yield",
                    serde_json::json!({
                        "target": "main",
                    }),
                ));
            },

            PrivateNode::FunctionCall { name, arguments, return_flg:_ } => {
                if name != "print" {
                    self.stack.push(Data::new(
                        "function_call",
                        serde_json::json!({
                            "target": &name
                        }),
                    ));
                }

                for arg in arguments {
                    self.analyze_node(arg);
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
