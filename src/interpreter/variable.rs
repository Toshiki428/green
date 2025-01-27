use std::collections::HashMap;
use crate::{
    common::types::{GreenValue, LiteralValue},
    error::{
        error_code::ErrorCode, error_context::ErrorContext, error_message::ErrorMessage
    },
};

#[derive(Debug)]
pub struct VariableManager {
    scopes: Vec<HashMap<String, GreenValue>>,
}

impl VariableManager {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn set_variable(&mut self, name: &String, value: &GreenValue) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.clone(), value.clone());
        }
    }

    pub fn get_variable(&mut self, name: &str) -> Result<LiteralValue, String> {
        for scope in self.scopes.iter().rev() {
            if let Some(variable) = scope.get(name) {
                return Ok(variable.value.clone());
            }
        }
        Err(ErrorMessage::global().get_error_message(
            ErrorContext::new(
                ErrorCode::Runtime007,
                None, None,
                vec![("variable", name)]
            )
        )?)
    }

    pub fn change_variable(&mut self, name: String, value: GreenValue) -> Result<(), String> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(variable) = scope.get_mut(&name) {
                if variable.value_type == value.value_type {
                    variable.value = value.value;
                    return Ok(())
                } else {
                    return Err(ErrorMessage::global().get_error_message(
                        ErrorContext::new(
                            ErrorCode::Runtime010,
                            None, None,
                            vec![
                                ("variable_type", &variable.value_type.to_string()),
                                ("value_type", &value.value_type.to_string()),
                                ("name", &name),
                            ],
                        )
                    )?);
                }
            }
        }
        Err(ErrorMessage::global().get_error_message(
            ErrorContext::new(
                ErrorCode::Runtime007,
                None, None,
                vec![("variable", &name)],
            )
        )?)
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }
}