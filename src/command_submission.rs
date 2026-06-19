use crate::task::{Task, TaskError, TaskId, TaskManager, TaskOutcome};

pub fn submit_command(manager: &mut TaskManager, command: &str) -> Result<TaskOutcome, TaskError> {
    manager.add_command(command)
}

pub trait CommandRunner {
    fn run_command(&mut self, task_id: TaskId, command: &str);
    fn cancel_command(&mut self, task_id: TaskId);
    fn select_task(&mut self, task_id: TaskId);
}

pub struct HostCommandController<R> {
    manager: TaskManager,
    runner: R,
}

impl<R> HostCommandController<R>
where
    R: CommandRunner,
{
    pub fn new(concurrency_limit: usize, runner: R) -> Result<Self, TaskError> {
        Ok(Self {
            manager: TaskManager::new(concurrency_limit)?,
            runner,
        })
    }

    pub fn add_command(&mut self, command: &str) -> Result<TaskOutcome, TaskError> {
        let outcome = submit_command(&mut self.manager, command)?;
        self.run_started_tasks(&outcome);
        self.select_outcome_task(&outcome);
        Ok(outcome)
    }

    pub fn select_task(&mut self, task_id: TaskId) -> Result<TaskOutcome, TaskError> {
        let outcome = self.manager.select_task(task_id)?;
        self.select_outcome_task(&outcome);
        Ok(outcome)
    }

    pub fn set_concurrency_limit(
        &mut self,
        concurrency_limit: usize,
    ) -> Result<TaskOutcome, TaskError> {
        let outcome = self.manager.set_concurrency_limit(concurrency_limit)?;
        self.run_started_tasks(&outcome);
        Ok(outcome)
    }

    pub fn mark_finished(
        &mut self,
        task_id: TaskId,
        exit_code: i32,
    ) -> Result<TaskOutcome, TaskError> {
        let outcome = self.manager.mark_finished(task_id, exit_code)?;
        self.run_started_tasks(&outcome);
        Ok(outcome)
    }

    pub fn cancel(&mut self, task_id: TaskId) -> Result<TaskOutcome, TaskError> {
        let was_running =
            self.manager.try_task(task_id)?.status() == &crate::task::TaskStatus::Running;
        let outcome = self.manager.cancel(task_id)?;
        if was_running {
            self.runner.cancel_command(task_id);
        }
        self.run_started_tasks(&outcome);
        Ok(outcome)
    }

    pub fn task(&self, task_id: TaskId) -> &Task {
        self.manager.task(task_id)
    }

    pub fn task_count(&self) -> usize {
        self.manager.task_count()
    }

    fn run_started_tasks(&mut self, outcome: &TaskOutcome) {
        let commands = outcome
            .tasks_to_start
            .iter()
            .map(|task_id| (*task_id, self.manager.task(*task_id).command().to_string()))
            .collect::<Vec<_>>();

        for (task_id, command) in commands {
            self.runner.run_command(task_id, &command);
        }
    }

    fn select_outcome_task(&mut self, outcome: &TaskOutcome) {
        if let Some(task_id) = outcome.selected_task_id {
            self.runner.select_task(task_id);
        }
    }
}
