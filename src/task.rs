pub use crate::task_error::TaskError;

pub type TaskId = usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Finished { exit_code: i32 },
    Canceled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    command: String,
    status: TaskStatus,
}

impl Task {
    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn status(&self) -> &TaskStatus {
        &self.status
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskOutcome {
    pub task_id: Option<TaskId>,
    pub selected_task_id: Option<TaskId>,
    pub tasks_to_start: Vec<TaskId>,
}

#[derive(Debug, Clone)]
pub struct TaskManager {
    concurrency_limit: usize,
    selected_task_id: Option<TaskId>,
    tasks: Vec<Task>,
}

impl TaskManager {
    pub fn new(concurrency_limit: usize) -> Result<Self, TaskError> {
        validate_concurrency_limit(concurrency_limit)?;

        Ok(Self {
            concurrency_limit,
            selected_task_id: None,
            tasks: Vec::new(),
        })
    }

    pub fn add_command(&mut self, command: &str) -> Result<TaskOutcome, TaskError> {
        validate_command(command)?;

        let task_id = self.tasks.len();
        let status = if self.running_count() < self.concurrency_limit {
            TaskStatus::Running
        } else {
            TaskStatus::Pending
        };
        self.tasks.push(Task {
            command: command.to_string(),
            status,
        });
        self.selected_task_id = Some(task_id);

        Ok(TaskOutcome::new(
            Some(task_id),
            Some(task_id),
            self.tasks_to_start_for_added_task(task_id),
        ))
    }

    pub fn select_task(&mut self, task_id: TaskId) -> Result<TaskOutcome, TaskError> {
        self.try_task(task_id)?;
        self.selected_task_id = Some(task_id);

        Ok(TaskOutcome::new(Some(task_id), Some(task_id), Vec::new()))
    }

    pub fn mark_finished(
        &mut self,
        task_id: TaskId,
        exit_code: i32,
    ) -> Result<TaskOutcome, TaskError> {
        if self.try_task(task_id)?.status != TaskStatus::Running {
            return Err(TaskError::TaskNotRunning(task_id));
        }

        self.tasks[task_id].status = TaskStatus::Finished { exit_code };
        Ok(TaskOutcome::new(
            Some(task_id),
            None,
            self.start_pending_tasks_to_capacity(),
        ))
    }

    pub fn cancel(&mut self, task_id: TaskId) -> Result<TaskOutcome, TaskError> {
        match self.try_task(task_id)?.status {
            TaskStatus::Pending => {
                self.tasks[task_id].status = TaskStatus::Canceled;
                Ok(TaskOutcome::new(Some(task_id), None, Vec::new()))
            }
            TaskStatus::Running => {
                self.tasks[task_id].status = TaskStatus::Canceled;
                Ok(TaskOutcome::new(
                    Some(task_id),
                    None,
                    self.start_pending_tasks_to_capacity(),
                ))
            }
            TaskStatus::Finished { .. } | TaskStatus::Canceled => {
                Err(TaskError::TaskNotCancelable(task_id))
            }
        }
    }

    pub fn set_concurrency_limit(
        &mut self,
        concurrency_limit: usize,
    ) -> Result<TaskOutcome, TaskError> {
        validate_concurrency_limit(concurrency_limit)?;

        self.concurrency_limit = concurrency_limit;
        Ok(TaskOutcome::new(
            None,
            None,
            self.start_pending_tasks_to_capacity(),
        ))
    }

    pub fn task(&self, task_id: TaskId) -> &Task {
        self.tasks
            .get(task_id)
            .expect("task() requires a task id returned by TaskManager")
    }

    pub fn try_task(&self, task_id: TaskId) -> Result<&Task, TaskError> {
        self.tasks
            .get(task_id)
            .ok_or(TaskError::TaskNotFound(task_id))
    }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn selected_task_id(&self) -> Option<TaskId> {
        self.selected_task_id
    }

    pub fn running_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|task| task.status == TaskStatus::Running)
            .count()
    }

    pub fn pending_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|task| task.status == TaskStatus::Pending)
            .count()
    }

    fn tasks_to_start_for_added_task(&self, task_id: TaskId) -> Vec<TaskId> {
        if self.tasks[task_id].status == TaskStatus::Running {
            vec![task_id]
        } else {
            Vec::new()
        }
    }

    fn start_pending_tasks_to_capacity(&mut self) -> Vec<TaskId> {
        let mut tasks_to_start = Vec::new();

        while self.running_count() < self.concurrency_limit {
            let Some(task_id) = self.oldest_pending_task_id() else {
                break;
            };
            self.tasks[task_id].status = TaskStatus::Running;
            tasks_to_start.push(task_id);
        }

        tasks_to_start
    }

    fn oldest_pending_task_id(&self) -> Option<TaskId> {
        self.tasks
            .iter()
            .position(|task| task.status == TaskStatus::Pending)
    }
}

impl TaskOutcome {
    fn new(
        task_id: Option<TaskId>,
        selected_task_id: Option<TaskId>,
        tasks_to_start: Vec<TaskId>,
    ) -> Self {
        Self {
            task_id,
            selected_task_id,
            tasks_to_start,
        }
    }
}

fn validate_concurrency_limit(concurrency_limit: usize) -> Result<(), TaskError> {
    if concurrency_limit == 0 {
        return Err(TaskError::InvalidConcurrencyLimit);
    }

    Ok(())
}

fn validate_command(command: &str) -> Result<(), TaskError> {
    if command.trim().is_empty() {
        return Err(TaskError::EmptyCommand);
    }

    Ok(())
}
