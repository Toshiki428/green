use crate::{error::{error_code::ErrorCode, error_context::ErrorContext}, parser::node::PrivateNode};
// use super::execute::
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum CoroutineStatus {
    /// 実行準備OK
    Ready,
    /// 実行中
    Running,
    /// 一時停止中
    Paused,
    /// 実行完了
    Completed,
}

#[derive(Debug, Clone)]
pub struct CoroutineTask {
    pub task_name: String,
    pub status: CoroutineStatus,
    pub current_position: usize,
    pub process: Vec<PrivateNode>,
}
impl CoroutineTask {
    pub fn new(task_name: &str, process: &PrivateNode) -> Self {
        let process = match process {
            PrivateNode::Block { block_type:_, statements } => {
                statements
            },
            _ => panic!("coroutine block error"),
        };
        Self {
            task_name: task_name.to_string(),
            status: CoroutineStatus::Ready,
            current_position: 0,
            process: process.clone(),
        }
    }

    pub fn step(&mut self) {
        self.current_position += 1;
        if self.current_position >= self.process.len() {
            self.status = CoroutineStatus::Completed;
        }
    }
}

pub struct CoroutineManager {
    coroutine_defs: HashMap<String, PrivateNode>,
    coroutine_tasks: HashMap<String, CoroutineTask>,
}
impl CoroutineManager {
    pub fn new() -> Self {
        Self {
            coroutine_defs: HashMap::new(),
            coroutine_tasks: HashMap::new(),
        }
    }

    pub fn add_def(&mut self, coroutine_name: &str, process: &PrivateNode) {
        self.coroutine_defs.insert(
            coroutine_name.to_string(),
            process.clone(),
        );
    }

    pub fn add_task(&mut self, task_name: &str, coroutine_name: &str) -> Result<(), ErrorContext> {
        if let Some(process) = self.coroutine_defs.get(coroutine_name) {
            let task = CoroutineTask::new(task_name, process);
            self.coroutine_tasks.insert(task_name.to_string(), task);
            Ok(())
        } else {
            Err(ErrorContext::new(
                ErrorCode::Runtime019,
                None, None,
                vec![("coroutine", coroutine_name)],
            ))
        }
    }

    pub fn get_task(&mut self, task_name: &str) -> Result<CoroutineTask, ErrorContext> {
        if let Some(task) = self.coroutine_tasks.get(task_name) {
            Ok(task.clone())
        } else {
            Err(ErrorContext::new(
                ErrorCode::Runtime019,
                None, None,
                vec![("coroutine", task_name)],
            ))
        }
    }

    pub fn set_task(&mut self, task_name: &str, task: CoroutineTask) {
        self.coroutine_tasks.insert(task_name.to_string(), task);
    }
}