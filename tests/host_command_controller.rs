use std::sync::{Arc, Mutex};

use command_drop::command_submission::{CommandRunner, HostCommandController};
use command_drop::task::{TaskError, TaskId, TaskStatus};

#[derive(Clone, Default)]
struct RecordingRunner {
    started: Arc<Mutex<Vec<(TaskId, String)>>>,
    selected: Arc<Mutex<Vec<TaskId>>>,
}

impl RecordingRunner {
    fn started_commands(&self) -> Vec<(TaskId, String)> {
        self.started
            .lock()
            .expect("recording runner lock should not be poisoned")
            .clone()
    }

    fn selected_tasks(&self) -> Vec<TaskId> {
        self.selected
            .lock()
            .expect("recording runner lock should not be poisoned")
            .clone()
    }
}

impl CommandRunner for RecordingRunner {
    fn run_command(&mut self, task_id: TaskId, command: &str) {
        self.started
            .lock()
            .expect("recording runner lock should not be poisoned")
            .push((task_id, command.to_string()));
    }

    fn cancel_command(&mut self, task_id: TaskId) {
        self.started
            .lock()
            .expect("recording runner lock should not be poisoned")
            .push((task_id, "<canceled>".to_string()));
    }

    fn select_task(&mut self, task_id: TaskId) {
        self.selected
            .lock()
            .expect("recording runner lock should not be poisoned")
            .push(task_id);
    }
}

#[test]
fn add_command_submits_to_task_manager_and_runs_started_task() {
    let runner = RecordingRunner::default();
    let mut controller =
        HostCommandController::new(2, runner.clone()).expect("positive limit is valid");

    let outcome = controller
        .add_command("echo from gui")
        .expect("non-empty command should be accepted");
    let task_id = outcome
        .task_id
        .expect("added command should report its task id");

    assert_eq!(outcome.tasks_to_start, vec![task_id]);
    assert_eq!(
        runner.started_commands(),
        vec![(task_id, "echo from gui".to_string())]
    );
    assert_eq!(controller.task(task_id).status(), &TaskStatus::Running);
}

#[test]
fn add_command_notifies_runner_to_display_added_task() {
    let runner = RecordingRunner::default();
    let mut controller =
        HostCommandController::new(1, runner.clone()).expect("positive limit is valid");

    let first = controller
        .add_command("sleep 1")
        .expect("first command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let second = controller
        .add_command("echo queued but selected")
        .expect("second command should be accepted")
        .task_id
        .expect("added command should report its task id");

    assert_eq!(
        runner.started_commands(),
        vec![(first, "sleep 1".to_string())]
    );
    assert_eq!(runner.selected_tasks(), vec![first, second]);
    assert_eq!(controller.task(second).status(), &TaskStatus::Pending);
}

#[test]
fn select_task_switches_existing_display_without_creating_or_starting_task() {
    let runner = RecordingRunner::default();
    let mut controller =
        HostCommandController::new(2, runner.clone()).expect("positive limit is valid");
    let first = controller
        .add_command("echo first")
        .expect("first command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let second = controller
        .add_command("echo second")
        .expect("second command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let started_before_click = runner.started_commands();

    let outcome = controller
        .select_task(first)
        .expect("existing task should be selectable");

    assert_eq!(outcome.task_id, Some(first));
    assert_eq!(outcome.selected_task_id, Some(first));
    assert_eq!(outcome.tasks_to_start, Vec::new());
    assert_eq!(runner.started_commands(), started_before_click);
    assert_eq!(runner.selected_tasks(), vec![first, second, first]);
    assert_eq!(controller.task_count(), 2);
    assert_eq!(controller.task(first).command(), "echo first");
    assert_eq!(controller.task(second).command(), "echo second");
}

#[test]
fn selecting_between_existing_and_added_tasks_notifies_runner_with_matching_task_ids() {
    let runner = RecordingRunner::default();
    let mut controller =
        HostCommandController::new(1, runner.clone()).expect("positive limit is valid");
    let first = controller
        .add_command("echo first")
        .expect("first command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let second = controller
        .add_command("echo second")
        .expect("second command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let third = controller
        .add_command("echo third")
        .expect("third command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let started_before_selection = runner.started_commands();

    controller
        .select_task(first)
        .expect("existing first task should be selectable");
    controller
        .select_task(third)
        .expect("added third task should be selectable");
    let outcome = controller
        .select_task(second)
        .expect("existing second task should be selectable");

    assert_eq!(outcome.task_id, Some(second));
    assert_eq!(outcome.selected_task_id, Some(second));
    assert_eq!(outcome.tasks_to_start, Vec::new());
    assert_eq!(runner.started_commands(), started_before_selection);
    assert_eq!(
        runner.selected_tasks(),
        vec![first, second, third, first, third, second]
    );
    assert_eq!(controller.task_count(), 3);
    assert_eq!(controller.task(first).command(), "echo first");
    assert_eq!(controller.task(second).command(), "echo second");
    assert_eq!(controller.task(third).command(), "echo third");
}

#[test]
fn select_task_rejects_missing_state_without_notifying_runner() {
    let runner = RecordingRunner::default();
    let mut controller =
        HostCommandController::new(1, runner.clone()).expect("positive limit is valid");
    let existing = controller
        .add_command("echo existing")
        .expect("command should be accepted")
        .task_id
        .expect("added command should report its task id");

    let result = controller.select_task(42);

    assert_eq!(result, Err(TaskError::TaskNotFound(42)));
    assert_eq!(runner.selected_tasks(), vec![existing]);
    assert_eq!(controller.task_count(), 1);
}

#[test]
fn add_command_queues_without_running_when_limit_is_reached() {
    let runner = RecordingRunner::default();
    let mut controller =
        HostCommandController::new(1, runner.clone()).expect("positive limit is valid");
    let first = controller
        .add_command("sleep 1")
        .expect("first command should be accepted")
        .task_id
        .expect("added command should report its task id");

    let queued = controller
        .add_command("echo queued")
        .expect("second command should be accepted");

    assert_eq!(
        runner.started_commands(),
        vec![(first, "sleep 1".to_string())]
    );
    assert_eq!(queued.tasks_to_start, Vec::new());
    assert_eq!(
        controller
            .task(
                queued
                    .task_id
                    .expect("queued command should report its task id")
            )
            .status(),
        &TaskStatus::Pending
    );
}

#[test]
fn increasing_limit_runs_waiting_tasks_through_same_runner() {
    let runner = RecordingRunner::default();
    let mut controller =
        HostCommandController::new(1, runner.clone()).expect("positive limit is valid");
    let first = controller
        .add_command("echo first")
        .expect("first command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let second = controller
        .add_command("echo second")
        .expect("second command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let third = controller
        .add_command("echo third")
        .expect("third command should be accepted")
        .task_id
        .expect("added command should report its task id");

    let outcome = controller
        .set_concurrency_limit(2)
        .expect("positive limit is valid");

    assert_eq!(outcome.tasks_to_start, vec![second]);
    assert_eq!(
        runner.started_commands(),
        vec![
            (first, "echo first".to_string()),
            (second, "echo second".to_string())
        ]
    );
    assert_eq!(controller.task(second).status(), &TaskStatus::Running);
    assert_eq!(controller.task(third).status(), &TaskStatus::Pending);
}

#[test]
fn increasing_limit_runs_each_waiting_task_up_to_new_capacity() {
    let runner = RecordingRunner::default();
    let mut controller =
        HostCommandController::new(1, runner.clone()).expect("positive limit is valid");
    let first = controller
        .add_command("echo first")
        .expect("first command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let second = controller
        .add_command("echo second")
        .expect("second command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let third = controller
        .add_command("echo third")
        .expect("third command should be accepted")
        .task_id
        .expect("added command should report its task id");

    let outcome = controller
        .set_concurrency_limit(3)
        .expect("positive limit is valid");

    assert_eq!(outcome.tasks_to_start, vec![second, third]);
    assert_eq!(
        runner.started_commands(),
        vec![
            (first, "echo first".to_string()),
            (second, "echo second".to_string()),
            (third, "echo third".to_string())
        ]
    );
    assert_eq!(controller.task(second).status(), &TaskStatus::Running);
    assert_eq!(controller.task(third).status(), &TaskStatus::Running);
}

#[test]
fn finishing_running_task_runs_oldest_waiting_task_through_same_runner() {
    let runner = RecordingRunner::default();
    let mut controller =
        HostCommandController::new(1, runner.clone()).expect("positive limit is valid");
    let first = controller
        .add_command("echo first")
        .expect("first command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let second = controller
        .add_command("echo second")
        .expect("second command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let third = controller
        .add_command("echo third")
        .expect("third command should be accepted")
        .task_id
        .expect("added command should report its task id");

    let outcome = controller
        .mark_finished(first, 0)
        .expect("running task should finish");

    assert_eq!(outcome.tasks_to_start, vec![second]);
    assert_eq!(
        runner.started_commands(),
        vec![
            (first, "echo first".to_string()),
            (second, "echo second".to_string())
        ]
    );
    assert_eq!(
        controller.task(first).status(),
        &TaskStatus::Finished { exit_code: 0 }
    );
    assert_eq!(controller.task(second).status(), &TaskStatus::Running);
    assert_eq!(controller.task(third).status(), &TaskStatus::Pending);
}

#[test]
fn finishing_second_running_task_updates_that_task_and_keeps_other_child_mapping_intact() {
    let runner = RecordingRunner::default();
    let mut controller =
        HostCommandController::new(2, runner.clone()).expect("positive limit is valid");
    let first = controller
        .add_command("echo first")
        .expect("first command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let second = controller
        .add_command("echo second")
        .expect("second command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let third = controller
        .add_command("echo third")
        .expect("third command should be accepted")
        .task_id
        .expect("added command should report its task id");

    let outcome = controller
        .mark_finished(second, 7)
        .expect("second running task should finish");

    assert_eq!(outcome.tasks_to_start, vec![third]);
    assert_eq!(
        runner.started_commands(),
        vec![
            (first, "echo first".to_string()),
            (second, "echo second".to_string()),
            (third, "echo third".to_string())
        ]
    );
    assert_eq!(controller.task(first).status(), &TaskStatus::Running);
    assert_eq!(
        controller.task(second).status(),
        &TaskStatus::Finished { exit_code: 7 }
    );
    assert_eq!(controller.task(third).status(), &TaskStatus::Running);
}

#[test]
fn canceling_running_task_cancels_runner_mapping_and_starts_next_waiting_task() {
    let runner = RecordingRunner::default();
    let mut controller =
        HostCommandController::new(1, runner.clone()).expect("positive limit is valid");
    let first = controller
        .add_command("sleep 1")
        .expect("first command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let second = controller
        .add_command("echo next")
        .expect("second command should be accepted")
        .task_id
        .expect("added command should report its task id");

    let outcome = controller
        .cancel(first)
        .expect("running task should be cancelable");

    assert_eq!(outcome.tasks_to_start, vec![second]);
    assert_eq!(
        runner.started_commands(),
        vec![
            (first, "sleep 1".to_string()),
            (first, "<canceled>".to_string()),
            (second, "echo next".to_string())
        ]
    );
    assert_eq!(controller.task(first).status(), &TaskStatus::Canceled);
    assert_eq!(controller.task(second).status(), &TaskStatus::Running);
}

#[test]
fn zero_concurrency_limit_is_rejected_at_controller_boundary() {
    let result = HostCommandController::new(0, RecordingRunner::default());

    assert!(matches!(result, Err(TaskError::InvalidConcurrencyLimit)));
}

#[test]
fn changing_limit_without_tasks_does_not_call_runner() {
    let runner = RecordingRunner::default();
    let mut controller =
        HostCommandController::new(1, runner.clone()).expect("positive limit is valid");

    let outcome = controller
        .set_concurrency_limit(2)
        .expect("positive limit is valid");

    assert_eq!(outcome.task_id, None);
    assert_eq!(outcome.tasks_to_start, Vec::new());
    assert_eq!(runner.started_commands(), Vec::new());
    assert_eq!(controller.task_count(), 0);
}

#[test]
fn blank_command_is_rejected_before_runner_is_called() {
    let runner = RecordingRunner::default();
    let mut controller =
        HostCommandController::new(1, runner.clone()).expect("positive limit is valid");

    let result = controller.add_command(" \t\n ");

    assert!(result.is_err());
    assert_eq!(runner.started_commands(), Vec::new());
    assert_eq!(controller.task_count(), 0);
}
