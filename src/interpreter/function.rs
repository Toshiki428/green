use std::collections::HashMap;

use crate::{common::types::Type, parser::node::PrivateNode};

pub struct FunctionManager {
    pub function_defs: HashMap<String, FunctionDef>,
}
impl FunctionManager {
    pub fn new() -> Self {
        Self { function_defs: HashMap::new() }
    }

    pub fn add_def(&mut self, function_name: &str, parameters: Vec<(String, Type)>, process: &PrivateNode) {
        self.function_defs.insert(
            function_name.to_string(),
            FunctionDef {
                name: function_name.to_string(),
                parameters,
                process: process.clone(),
            }
        );
    }

    pub fn get_function(&self, function_name: &str) -> Option<&FunctionDef> {
        self.function_defs.get(function_name)
    }
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub parameters: Vec<(String, Type)>,
    pub process: PrivateNode,
}
