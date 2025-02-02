use std::collections::HashMap;
use crate::{common::types::Type, parser::node::ParameterNode};
use super::variable_table::VariableScope;

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    /// 関数名
    pub name: String,

    /// パラメータ
    pub parameters: Vec<ParameterNode>,
    
    /// 戻り値
    pub return_type: Option<Type>,

    /// 関数内の変数
    pub local_variables: VariableScope,

    /// 可変長引数であるか  
    /// trueであれば、引数の数と型をチェックしない
    pub is_variadic: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionTable {
    table: HashMap<String, FunctionInfo>,
}
impl FunctionTable {
    pub fn new() -> Self {
        let mut table = Self {
            table: HashMap::new(),
        };
        table.function_definition("print", &vec![], &None, true);
        return table
    }

    pub fn function_definition(&mut self, name: &str, parameters: &Vec<ParameterNode>, return_type: &Option<Type>, is_variadic: bool) {
        let function_info = FunctionInfo {
            name: name.to_string(),
            parameters: parameters.clone(),
            return_type: return_type.clone(),
            local_variables: VariableScope::new(None),
            is_variadic,
        };
        self.table.insert(name.to_string(), function_info);
    }

    pub fn get_function_info(&self, name: &str) -> Option<FunctionInfo> {
        self.table.get(name).cloned()
    }

    pub fn get_function_info_mut(&mut self, name: &str) -> Option<&mut FunctionInfo> {
        self.table.get_mut(name)
    }
}