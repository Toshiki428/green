use crate::parser::node::{BlockNode, PrivateNode};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum TaskStatus {
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
    pub coroutine_name: String,
    pub status: TaskStatus,
    pub current_position: usize,
    pub process: Vec<PrivateNode>,
}
impl CoroutineTask {
    pub fn new(task_name: &str, coroutine_name: &str, process: &BlockNode) -> Self {
        let process = process.statements.clone();
        Self {
            task_name: task_name.to_string(),
            coroutine_name: coroutine_name.to_string(),
            status: TaskStatus::Ready,
            current_position: 0,
            process,
        }
    }

    pub fn step(&mut self) {
        self.current_position += 1;
        if self.current_position >= self.process.len() {
            self.status = TaskStatus::Completed;
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskTable {
    pub table: HashMap<String, CoroutineTask>,
}
impl TaskTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    /// タスクの新規登録
    pub fn add_task(&mut self, task_name: &str, coroutine_name: &str, process: &BlockNode) {
        let task = CoroutineTask::new(task_name, coroutine_name, process);
        self.table.insert(task_name.to_string(), task);
    }

    /// タスクの上書き
    pub fn set_task(&mut self, task_name: &str, task: CoroutineTask) {
        self.table.insert(task_name.to_string(), task);
    }

    pub fn get_task(&mut self, task_name: &str) -> Option<CoroutineTask> {
        if let Some(task) = self.table.get(task_name) {
            Some(task.clone())
        } else {
            None
        }
    }
}