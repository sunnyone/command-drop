use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::task::TaskId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskError {
    InvalidConcurrencyLimit,
    EmptyCommand,
    TaskNotFound(TaskId),
    TaskNotRunning(TaskId),
    TaskNotCancelable(TaskId),
}

impl Display for TaskError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidConcurrencyLimit => write!(f, "concurrency limit must be positive"),
            Self::EmptyCommand => write!(f, "command must not be empty"),
            Self::TaskNotFound(task_id) => write!(f, "task not found: {task_id}"),
            Self::TaskNotRunning(task_id) => write!(f, "task is not running: {task_id}"),
            Self::TaskNotCancelable(task_id) => write!(f, "task is not cancelable: {task_id}"),
        }
    }
}

impl Error for TaskError {}
