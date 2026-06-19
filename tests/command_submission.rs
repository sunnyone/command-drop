use command_drop::command_submission::submit_command;
use command_drop::task::{TaskManager, TaskStatus};

#[test]
fn ui_submission_uses_shared_task_addition_path() {
    let mut manager = TaskManager::new(2).expect("positive limit is valid");

    let outcome =
        submit_command(&mut manager, "echo from ui").expect("UI command should be accepted");
    let task_id = outcome
        .task_id
        .expect("submitted command should report its task id");

    assert_eq!(outcome.tasks_to_start, vec![task_id]);
    assert_eq!(outcome.selected_task_id, Some(task_id));
    assert_eq!(manager.selected_task_id(), Some(task_id));
    assert_eq!(manager.task(task_id).command(), "echo from ui");
    assert_eq!(manager.task(task_id).status(), &TaskStatus::Running);
}

#[test]
fn dbus_submission_uses_same_validation_as_ui_submission() {
    let mut manager = TaskManager::new(2).expect("positive limit is valid");

    let outcome =
        submit_command(&mut manager, "echo from dbus").expect("DBus command should be accepted");
    let task_id = outcome
        .task_id
        .expect("submitted command should report its task id");

    assert_eq!(outcome.tasks_to_start, vec![task_id]);
    assert_eq!(outcome.selected_task_id, Some(task_id));
    assert_eq!(manager.selected_task_id(), Some(task_id));
    assert_eq!(manager.task(task_id).command(), "echo from dbus");
    assert_eq!(manager.task(task_id).status(), &TaskStatus::Running);
}

#[test]
fn shared_submission_path_rejects_empty_command_before_task_creation() {
    let mut manager = TaskManager::new(2).expect("positive limit is valid");

    let ui_result = submit_command(&mut manager, "");
    let dbus_result = submit_command(&mut manager, " \t\n ");

    assert!(ui_result.is_err());
    assert!(dbus_result.is_err());
    assert_eq!(manager.task_count(), 0);
}
