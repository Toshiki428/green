use std::collections::HashMap;
use crate::parser::node::BlockNode;

use super::variable_table::VariableScope;

#[derive(Debug, Clone)]
pub struct CoroutineInfo {
    /// コルーチン名
    pub name: String,

    /// Docコメント
    pub doc: String,

    // /// パラメータ
    // pub parameters: Vec<ParameterNode>,
    
    // /// 戻り値
    // pub return_type: Option<Type>,

    /// コルーチン内の変数
    pub local_variables: VariableScope,

    /// コルーチンの処理
    pub process: BlockNode,
}

#[derive(Debug, Clone)]
pub struct CoroutineTable {
    table: HashMap<String, CoroutineInfo>,
}
impl CoroutineTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub fn coroutine_definition(&mut self, name: &str, doc: Option<&str>, process: &BlockNode) {
        let doc = match doc {
            Some(doc) => doc,
            None => "",
        };
        
        let coroutine_info = CoroutineInfo {
            name: name.to_string(),
            doc: doc.to_string(),
            local_variables: VariableScope::new(None),
            process: process.clone(),
        };
        self.table.insert(name.to_string(), coroutine_info);
    }

    pub fn get_coroutine_info(&self, name: &str) -> Option<CoroutineInfo> {
        self.table.get(name).cloned()
    }

    pub fn get_coroutine_info_mut(&mut self, name: &str) -> Option<&mut CoroutineInfo> {
        self.table.get_mut(name)
    }
}