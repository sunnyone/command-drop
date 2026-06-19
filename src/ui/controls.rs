use gtk4::prelude::*;

use crate::task::{TaskError, TaskManager};

use super::host_state::{sync_outcome_tasks, GuiController, GuiTaskList};

pub(crate) fn build_controls(controller: GuiController, task_list: GuiTaskList) -> gtk4::Box {
    let controls = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    let command_entry = gtk4::Entry::builder().hexpand(true).build();
    let add_button = gtk4::Button::with_label("Add");
    let limit_entry = gtk4::Entry::builder().text("1").width_chars(4).build();

    connect_add_button(
        &add_button,
        &command_entry,
        controller.clone(),
        task_list.clone(),
    );
    connect_command_entry(&command_entry, controller.clone(), task_list.clone());
    connect_limit_entry(&limit_entry, controller, task_list);

    controls.append(&command_entry);
    controls.append(&add_button);
    controls.append(&limit_entry);
    controls
}

fn connect_add_button(
    add_button: &gtk4::Button,
    command_entry: &gtk4::Entry,
    controller: GuiController,
    task_list: GuiTaskList,
) {
    let command_entry = command_entry.clone();
    add_button.connect_clicked(move |_| {
        submit_command_entry(&command_entry, controller.clone(), task_list.clone());
    });
}

fn connect_command_entry(
    command_entry: &gtk4::Entry,
    controller: GuiController,
    task_list: GuiTaskList,
) {
    command_entry.connect_activate(move |entry| {
        submit_command_entry(entry, controller.clone(), task_list.clone());
    });
}

fn submit_command_entry(
    command_entry: &gtk4::Entry,
    controller: GuiController,
    task_list: GuiTaskList,
) {
    let command = command_entry.text().to_string();
    let result = controller.borrow_mut().add_command(&command);
    match result {
        Ok(outcome) => {
            sync_outcome_tasks(&controller, &task_list, outcome);
            command_entry.set_text("");
        }
        Err(error) => eprintln!("rejected command: {error}"),
    }
}

fn connect_limit_entry(
    limit_entry: &gtk4::Entry,
    controller: GuiController,
    task_list: GuiTaskList,
) {
    limit_entry.connect_activate(move |entry| {
        let limit = match parse_concurrency_limit(entry.text().as_str()) {
            Ok(limit) => limit,
            Err(error) => {
                eprintln!("rejected concurrency limit: {error}");
                return;
            }
        };

        let result = controller.borrow_mut().set_concurrency_limit(limit);
        match result {
            Ok(outcome) => sync_outcome_tasks(&controller, &task_list, outcome),
            Err(error) => eprintln!("rejected concurrency limit: {error}"),
        }
    });
}

fn parse_concurrency_limit(raw: &str) -> Result<usize, TaskError> {
    match raw.trim().parse::<usize>() {
        Ok(limit) => TaskManager::new(limit).map(|_| limit),
        Err(_) => Err(TaskError::InvalidConcurrencyLimit),
    }
}

#[cfg(test)]
mod tests {
    use crate::task::TaskError;

    #[test]
    fn rejects_zero_concurrency_limit_at_control_boundary() {
        assert_eq!(
            super::parse_concurrency_limit("0"),
            Err(TaskError::InvalidConcurrencyLimit)
        );
    }
}
