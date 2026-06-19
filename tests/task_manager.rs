use command_drop::task::{TaskError, TaskManager, TaskStatus};

#[test]
fn add_command_starts_immediately_when_below_limit() {
    let mut manager = TaskManager::new(2).expect("positive limit is valid");

    let outcome = manager
        .add_command("echo one")
        .expect("non-empty command should be accepted");
    let task_id = outcome
        .task_id
        .expect("added command should report its task id");

    assert_eq!(outcome.tasks_to_start, vec![task_id]);
    assert_eq!(manager.task(task_id).status(), &TaskStatus::Running);
    assert_eq!(manager.running_count(), 1);
    assert_eq!(manager.pending_count(), 0);
}

#[test]
fn add_command_queues_when_limit_is_reached() {
    let mut manager = TaskManager::new(1).expect("positive limit is valid");
    let running = manager
        .add_command("sleep 1")
        .expect("first command should be accepted")
        .task_id
        .expect("added command should report its task id");

    let queued = manager
        .add_command("echo queued")
        .expect("second command should be accepted");

    assert_eq!(manager.task(running).status(), &TaskStatus::Running);
    assert_eq!(queued.tasks_to_start, Vec::new());
    assert_eq!(
        manager
            .task(
                queued
                    .task_id
                    .expect("queued command should report its task id")
            )
            .status(),
        &TaskStatus::Pending
    );
    assert_eq!(manager.running_count(), 1);
    assert_eq!(manager.pending_count(), 1);
}

#[test]
fn new_manager_starts_without_selected_task() {
    let manager = TaskManager::new(1).expect("positive limit is valid");

    assert_eq!(manager.selected_task_id(), None);
}

#[test]
fn add_command_creates_task_state_and_selects_added_task() {
    let mut manager = TaskManager::new(1).expect("positive limit is valid");
    let first = manager
        .add_command("sleep 1")
        .expect("first command should be accepted")
        .task_id
        .expect("added command should report its task id");

    let outcome = manager
        .add_command("echo selected even while pending")
        .expect("second command should be accepted");
    let second = outcome
        .task_id
        .expect("added command should report its task id");

    assert_eq!(manager.task_count(), 2);
    assert_eq!(manager.task(first).status(), &TaskStatus::Running);
    assert_eq!(manager.task(second).status(), &TaskStatus::Pending);
    assert_eq!(outcome.selected_task_id, Some(second));
    assert_eq!(manager.selected_task_id(), Some(second));
}

#[test]
fn select_task_switches_to_existing_task_without_creating_state() {
    let mut manager = TaskManager::new(2).expect("positive limit is valid");
    let first = manager
        .add_command("echo first")
        .expect("first command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let second = manager
        .add_command("echo second")
        .expect("second command should be accepted")
        .task_id
        .expect("added command should report its task id");

    let outcome = manager
        .select_task(first)
        .expect("existing task should be selectable");

    assert_eq!(outcome.task_id, Some(first));
    assert_eq!(outcome.selected_task_id, Some(first));
    assert_eq!(outcome.tasks_to_start, Vec::new());
    assert_eq!(manager.selected_task_id(), Some(first));
    assert_eq!(manager.task_count(), 2);
    assert_eq!(manager.task(first).command(), "echo first");
    assert_eq!(manager.task(second).command(), "echo second");
}

#[test]
fn selecting_between_existing_and_added_tasks_keeps_selection_on_matching_state() {
    let mut manager = TaskManager::new(1).expect("positive limit is valid");
    let first = manager
        .add_command("echo first")
        .expect("first command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let second = manager
        .add_command("echo second")
        .expect("second command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let third = manager
        .add_command("echo third")
        .expect("third command should be accepted")
        .task_id
        .expect("added command should report its task id");

    let first_selection = manager
        .select_task(first)
        .expect("existing first task should be selectable");
    let third_selection = manager
        .select_task(third)
        .expect("added third task should be selectable");
    let second_selection = manager
        .select_task(second)
        .expect("existing second task should be selectable");

    assert_eq!(first_selection.selected_task_id, Some(first));
    assert_eq!(third_selection.selected_task_id, Some(third));
    assert_eq!(second_selection.selected_task_id, Some(second));
    assert_eq!(manager.selected_task_id(), Some(second));
    assert_eq!(manager.task_count(), 3);
    assert_eq!(manager.task(first).command(), "echo first");
    assert_eq!(manager.task(second).command(), "echo second");
    assert_eq!(manager.task(third).command(), "echo third");
}

#[test]
fn select_task_rejects_missing_state_and_preserves_current_selection() {
    let mut manager = TaskManager::new(1).expect("positive limit is valid");
    let existing = manager
        .add_command("echo existing")
        .expect("command should be accepted")
        .task_id
        .expect("added command should report its task id");

    let result = manager.select_task(42);

    assert_eq!(result, Err(TaskError::TaskNotFound(42)));
    assert_eq!(manager.selected_task_id(), Some(existing));
    assert_eq!(manager.task_count(), 1);
}

#[test]
fn finishing_running_task_starts_oldest_pending_task() {
    let mut manager = TaskManager::new(1).expect("positive limit is valid");
    let first = manager
        .add_command("echo first")
        .expect("first command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let second = manager
        .add_command("echo second")
        .expect("second command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let third = manager
        .add_command("echo third")
        .expect("third command should be accepted")
        .task_id
        .expect("added command should report its task id");

    let outcome = manager
        .mark_finished(first, 0)
        .expect("running task can finish");

    assert_eq!(outcome.tasks_to_start, vec![second]);
    assert_eq!(
        manager.task(first).status(),
        &TaskStatus::Finished { exit_code: 0 }
    );
    assert_eq!(manager.task(second).status(), &TaskStatus::Running);
    assert_eq!(manager.task(third).status(), &TaskStatus::Pending);
    assert_eq!(manager.running_count(), 1);
    assert_eq!(manager.pending_count(), 1);
}

#[test]
fn increasing_limit_starts_waiting_tasks_up_to_new_capacity() {
    let mut manager = TaskManager::new(1).expect("positive limit is valid");
    let first = manager
        .add_command("echo first")
        .expect("first command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let second = manager
        .add_command("echo second")
        .expect("second command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let third = manager
        .add_command("echo third")
        .expect("third command should be accepted")
        .task_id
        .expect("added command should report its task id");

    let outcome = manager
        .set_concurrency_limit(2)
        .expect("positive limit is valid");

    assert_eq!(outcome.tasks_to_start, vec![second]);
    assert_eq!(manager.task(first).status(), &TaskStatus::Running);
    assert_eq!(manager.task(second).status(), &TaskStatus::Running);
    assert_eq!(manager.task(third).status(), &TaskStatus::Pending);
    assert_eq!(manager.running_count(), 2);
    assert_eq!(manager.pending_count(), 1);
}

#[test]
fn changing_limit_without_tasks_reports_no_changed_task() {
    let mut manager = TaskManager::new(1).expect("positive limit is valid");

    let outcome = manager
        .set_concurrency_limit(2)
        .expect("positive limit is valid");

    assert_eq!(outcome.task_id, None);
    assert_eq!(outcome.tasks_to_start, Vec::new());
    assert_eq!(manager.task_count(), 0);
}

#[test]
fn empty_or_whitespace_command_is_rejected() {
    let mut manager = TaskManager::new(1).expect("positive limit is valid");

    let empty = manager.add_command("");
    let whitespace = manager.add_command(" \t\n ");

    assert!(empty.is_err());
    assert!(whitespace.is_err());
    assert_eq!(manager.task_count(), 0);
}

#[test]
fn zero_concurrency_limit_is_rejected_at_boundary() {
    let created = TaskManager::new(0);

    assert!(created.is_err());
}

#[test]
fn finishing_unknown_task_returns_task_not_found() {
    let mut manager = TaskManager::new(1).expect("positive limit is valid");

    let result = manager.mark_finished(42, 0);

    assert_eq!(result, Err(TaskError::TaskNotFound(42)));
}

#[test]
fn canceling_unknown_task_returns_task_not_found() {
    let mut manager = TaskManager::new(1).expect("positive limit is valid");

    let result = manager.cancel(42);

    assert_eq!(result, Err(TaskError::TaskNotFound(42)));
}

#[test]
fn canceling_running_task_releases_capacity_for_oldest_pending_task() {
    let mut manager = TaskManager::new(1).expect("positive limit is valid");
    let running = manager
        .add_command("sleep 1")
        .expect("first command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let pending = manager
        .add_command("echo after cancel")
        .expect("second command should be accepted")
        .task_id
        .expect("added command should report its task id");

    let outcome = manager
        .cancel(running)
        .expect("running task should be cancelable");

    assert_eq!(outcome.tasks_to_start, vec![pending]);
    assert_eq!(manager.task(running).status(), &TaskStatus::Canceled);
    assert_eq!(manager.task(pending).status(), &TaskStatus::Running);
    assert_eq!(manager.running_count(), 1);
    assert_eq!(manager.pending_count(), 0);
}

#[test]
fn canceling_pending_task_does_not_start_another_task() {
    let mut manager = TaskManager::new(1).expect("positive limit is valid");
    let running = manager
        .add_command("sleep 1")
        .expect("first command should be accepted")
        .task_id
        .expect("added command should report its task id");
    let pending = manager
        .add_command("echo pending")
        .expect("second command should be accepted")
        .task_id
        .expect("added command should report its task id");

    let outcome = manager
        .cancel(pending)
        .expect("pending task should be cancelable");

    assert_eq!(outcome.tasks_to_start, Vec::new());
    assert_eq!(manager.task(running).status(), &TaskStatus::Running);
    assert_eq!(manager.task(pending).status(), &TaskStatus::Canceled);
    assert_eq!(manager.running_count(), 1);
    assert_eq!(manager.pending_count(), 0);
}

#[test]
fn finished_task_is_not_cancelable() {
    let mut manager = TaskManager::new(1).expect("positive limit is valid");
    let task_id = manager
        .add_command("echo done")
        .expect("command should be accepted")
        .task_id
        .expect("added command should report its task id");
    manager
        .mark_finished(task_id, 0)
        .expect("running task should finish");

    let result = manager.cancel(task_id);

    assert_eq!(result, Err(TaskError::TaskNotCancelable(task_id)));
}
