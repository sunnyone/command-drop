use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;

use gtk4::prelude::*;

use crate::command_submission::HostCommandController;
use crate::terminal::TerminalPane;

use super::controls::build_controls;
use super::host_state::{poll_dbus_commands, poll_finished_tasks, GuiController, GuiTaskList};
use super::task_list::TaskList;

const DEFAULT_WINDOW_WIDTH: i32 = 1100;
const INITIAL_LEFT_PANE_WIDTH: i32 = DEFAULT_WINDOW_WIDTH * 3 / 10;

pub(crate) fn build_window(app: &gtk4::Application, receiver: mpsc::Receiver<String>) {
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("Command Drop")
        .default_width(DEFAULT_WINDOW_WIDTH)
        .default_height(720)
        .build();

    let root = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    root.set_hexpand(true);
    root.set_vexpand(true);
    let (finished_sender, finished_receiver) = mpsc::channel();
    let terminal = TerminalPane::new(finished_sender);
    let terminal_widget = terminal.widget().clone();
    let controller = build_controller(terminal);
    let task_list = Rc::new(RefCell::new(TaskList::new()));
    let controls = build_controls(controller.clone(), task_list.clone());
    let panes = build_panes(&terminal_widget, controller.clone(), task_list.clone());
    poll_dbus_commands(receiver, controller.clone(), task_list.clone());
    poll_finished_tasks(finished_receiver, controller, task_list);

    root.append(&controls);
    root.append(&panes);
    window.set_child(Some(&root));
    window.present();
}

fn build_controller(terminal: TerminalPane) -> GuiController {
    Rc::new(RefCell::new(
        HostCommandController::new(1, terminal).expect("default concurrency limit is valid"),
    ))
}

fn build_panes(
    terminal: &gtk4::Box,
    controller: GuiController,
    task_list: GuiTaskList,
) -> gtk4::Paned {
    let panes = gtk4::Paned::new(gtk4::Orientation::Horizontal);
    panes.set_hexpand(true);
    panes.set_vexpand(true);
    panes.set_resize_start_child(true);
    panes.set_resize_end_child(true);
    panes.set_shrink_start_child(true);
    panes.set_shrink_end_child(true);

    panes.set_start_child(Some(task_list.borrow().widget()));
    panes.set_end_child(Some(terminal));
    panes.set_position(INITIAL_LEFT_PANE_WIDTH);
    connect_task_selection(controller, task_list);
    panes
}

fn connect_task_selection(controller: GuiController, task_list: GuiTaskList) {
    let list = task_list.borrow().list().clone();
    list.connect_row_selected(move |_, row| {
        if task_list.borrow().is_syncing_selection() {
            return;
        }
        let Some(row) = row else {
            return;
        };
        let task_id = task_list
            .borrow()
            .task_id_for_row(row)
            .expect("selected list row must reference an existing task state");

        let result = controller.borrow_mut().select_task(task_id);
        if let Err(error) = result {
            eprintln!("failed to select task {task_id}: {error}");
        }
    });
}
