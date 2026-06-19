use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk4::prelude::*;

use crate::task::{Task, TaskId, TaskStatus};

pub(crate) struct TaskList {
    scroller: gtk4::ScrolledWindow,
    list: gtk4::ListBox,
    syncing_selection: Rc<Cell<bool>>,
    rows: HashMap<TaskId, TaskRow>,
}

struct TaskRow {
    row: gtk4::ListBoxRow,
    label: gtk4::Label,
}

impl TaskList {
    pub(crate) fn new() -> Self {
        let list = gtk4::ListBox::new();
        list.set_hexpand(true);
        list.set_vexpand(true);

        let scroller = gtk4::ScrolledWindow::new();
        scroller.set_hexpand(true);
        scroller.set_vexpand(true);
        scroller.set_child(Some(&list));

        Self {
            scroller,
            list,
            syncing_selection: Rc::new(Cell::new(false)),
            rows: HashMap::new(),
        }
    }

    pub(crate) fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.scroller
    }

    pub(crate) fn list(&self) -> &gtk4::ListBox {
        &self.list
    }

    pub(crate) fn sync_task(&mut self, task_id: TaskId, task: &Task) {
        let text = task_row_text(task_id, task);
        if let Some(task_row) = self.rows.get(&task_id) {
            task_row.label.set_text(&text);
            return;
        }

        let label = gtk4::Label::builder()
            .xalign(0.0)
            .label(text.as_str())
            .build();
        let row = gtk4::ListBoxRow::new();
        row.set_child(Some(&label));
        self.list.append(&row);
        self.rows.insert(task_id, TaskRow { row, label });
    }

    pub(crate) fn select_task(&self, task_id: TaskId) {
        let task_row = self
            .rows
            .get(&task_id)
            .expect("selected task must have a synced list row");
        self.syncing_selection.set(true);
        self.list.select_row(Some(&task_row.row));
        self.syncing_selection.set(false);
    }

    pub(crate) fn is_syncing_selection(&self) -> bool {
        self.syncing_selection.get()
    }

    pub(crate) fn task_id_for_row(&self, selected_row: &gtk4::ListBoxRow) -> Option<TaskId> {
        self.rows
            .iter()
            .find(|(_, task_row)| task_row.row == *selected_row)
            .map(|(task_id, _)| *task_id)
    }
}

fn task_row_text(task_id: TaskId, task: &Task) -> String {
    format!(
        "#{task_id} [{}] {}",
        task_status_text(task.status()),
        task.command()
    )
}

fn task_status_text(status: &TaskStatus) -> String {
    match status {
        TaskStatus::Pending => "pending".to_string(),
        TaskStatus::Running => "running".to_string(),
        TaskStatus::Finished { exit_code } => format!("finished:{exit_code}"),
        TaskStatus::Canceled => "canceled".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::task::TaskStatus;

    #[test]
    fn status_text_includes_terminal_states() {
        assert_eq!(super::task_status_text(&TaskStatus::Pending), "pending");
        assert_eq!(super::task_status_text(&TaskStatus::Running), "running");
        assert_eq!(
            super::task_status_text(&TaskStatus::Finished { exit_code: 7 }),
            "finished:7"
        );
        assert_eq!(super::task_status_text(&TaskStatus::Canceled), "canceled");
    }
}
