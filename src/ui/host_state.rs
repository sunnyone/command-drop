use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;

use crate::command_submission::HostCommandController;
use crate::task::{TaskId, TaskOutcome};
use crate::terminal::TerminalPane;

use super::task_list::TaskList;

pub(crate) type GuiController = Rc<RefCell<HostCommandController<TerminalPane>>>;
pub(crate) type GuiTaskList = Rc<RefCell<TaskList>>;

pub(crate) fn poll_dbus_commands(
    receiver: mpsc::Receiver<String>,
    controller: GuiController,
    task_list: GuiTaskList,
) {
    gtk4::glib::timeout_add_local(Duration::from_millis(50), move || {
        for command in receiver.try_iter() {
            let result = controller.borrow_mut().add_command(&command);
            match result {
                Ok(outcome) => sync_outcome_tasks(&controller, &task_list, outcome),
                Err(error) => eprintln!("rejected command: {error}"),
            }
        }

        gtk4::glib::ControlFlow::Continue
    });
}

pub(crate) fn poll_finished_tasks(
    receiver: mpsc::Receiver<(TaskId, i32)>,
    controller: GuiController,
    task_list: GuiTaskList,
) {
    gtk4::glib::timeout_add_local(Duration::from_millis(50), move || {
        for (task_id, exit_code) in receiver.try_iter() {
            let result = controller.borrow_mut().mark_finished(task_id, exit_code);
            match result {
                Ok(outcome) => sync_outcome_tasks(&controller, &task_list, outcome),
                Err(error) => eprintln!("failed to finish task {task_id}: {error}"),
            }
        }

        gtk4::glib::ControlFlow::Continue
    });
}

pub(crate) fn sync_outcome_tasks(
    controller: &GuiController,
    task_list: &GuiTaskList,
    outcome: TaskOutcome,
) {
    let controller = controller.borrow();
    for task_id in task_ids_to_sync(&outcome) {
        task_list
            .borrow_mut()
            .sync_task(task_id, controller.task(task_id));
    }
    if let Some(task_id) = outcome.selected_task_id {
        task_list.borrow().select_task(task_id);
    }
}

fn task_ids_to_sync(outcome: &TaskOutcome) -> Vec<TaskId> {
    let mut task_ids = Vec::new();
    if let Some(task_id) = outcome.task_id {
        task_ids.push(task_id);
    }
    task_ids.extend(outcome.tasks_to_start.iter().copied());
    task_ids
}

#[cfg(test)]
mod tests {
    #[test]
    fn limit_change_without_tasks_has_no_gui_sync_targets() {
        let outcome = crate::task::TaskOutcome {
            task_id: None,
            selected_task_id: None,
            tasks_to_start: Vec::new(),
        };

        assert_eq!(super::task_ids_to_sync(&outcome), Vec::new());
    }
}
