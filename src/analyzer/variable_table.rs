use crate::common::types::Type;

#[derive(Debug, Clone)]
pub struct VariableInfo {
    name: String,
    variable_type: Type,
}

#[derive(Debug, Clone)]
pub struct VariableScope {
    variable_info: Vec<VariableInfo>,
    parent_pointer: Option<usize>,
}
impl VariableScope {
    pub fn new(parent_pointer: Option<usize>) -> Self {
        Self {
            variable_info: Vec::new(),
            parent_pointer,
        }
    }

    /// 変数定義
    pub fn variable_declare(&mut self, name: &str, var_type: &Type) {
        self.variable_info.push(
            VariableInfo {
                name: name.to_string(),
                variable_type: var_type.clone(),
            }
        );
    }

    /// 変数呼び出し（型情報を返す）
    pub fn get_type(&self, name: &str) -> Option<Type> {
        for variable in &self.variable_info {
            if variable.name == name {
                return Some(variable.variable_type.clone())
            }
        }

        return None
    }
}

#[derive(Debug, Clone)]
pub struct  VariableTable {
    pub table: Vec<VariableScope>,
    /// 現在のポインタ
    current_pointer: usize,
    /// ポインタの最大値
    max_pointer: usize,
}
impl VariableTable {
    pub fn new() -> Self {
        Self {
            table: vec![VariableScope::new(None)],
            current_pointer: 0,
            max_pointer: 0,
        }
    }

    /// 変数宣言
    pub fn variable_declare(&mut self, name: &str, var_type: &Type) {
        self.table[self.current_pointer].variable_declare(name, var_type);
    }

    /// 変数の型を取得
    pub fn get_type(&mut self, name: &str) -> Option<Type> {
        let mut parent_pointer = self.table[self.current_pointer].parent_pointer;
        let mut current_pointer = self.current_pointer;
        loop {
            match self.table[current_pointer].get_type(name) {
                Some(variable_type) => return Some(variable_type),
                None => {
                    match parent_pointer {
                        Some(pointer) => {
                            current_pointer = pointer;
                            parent_pointer = self.table[current_pointer].parent_pointer;
                        },
                        None => return None,
                    }
                }
            }
        }
    }

    pub fn push_scope(&mut self, parent_pointer: Option<usize>) {
        self.table.push(VariableScope::new(parent_pointer));
        self.max_pointer += 1;
        self.current_pointer = self.max_pointer;
    }

    pub fn pop_scope(&mut self) {
        let current_scope = self.table.get(self.current_pointer);
        match current_scope {
            Some(scope) => {
                self.current_pointer = match scope.parent_pointer {
                    Some(pointer) => pointer,
                    None => panic!("pop_scopeができない")
                };
            },
            None => panic!("スコープが見つからない"),
        }
    }
}