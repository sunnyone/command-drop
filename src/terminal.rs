#[cfg(feature = "gui")]
pub struct TerminalPane {
    finished_sender: std::sync::mpsc::Sender<(TaskId, i32)>,
    container: gtk4::Box,
    terminal_widgets:
        std::rc::Rc<std::cell::RefCell<std::collections::HashMap<TaskId, vte4::Terminal>>>,
    running_terminals:
        std::rc::Rc<std::cell::RefCell<std::collections::HashMap<TaskId, gtk4::gio::Cancellable>>>,
    selected_task_id: std::rc::Rc<std::cell::RefCell<Option<TaskId>>>,
}

#[cfg(feature = "gui")]
impl TerminalPane {
    pub fn new(finished_sender: std::sync::mpsc::Sender<(TaskId, i32)>) -> Self {
        use gtk4::prelude::*;

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        container.set_hexpand(true);
        container.set_vexpand(true);

        Self {
            finished_sender,
            container,
            terminal_widgets: std::rc::Rc::new(std::cell::RefCell::new(
                std::collections::HashMap::new(),
            )),
            running_terminals: std::rc::Rc::new(std::cell::RefCell::new(
                std::collections::HashMap::new(),
            )),
            selected_task_id: std::rc::Rc::new(std::cell::RefCell::new(None)),
        }
    }

    pub fn widget(&self) -> &gtk4::Box {
        &self.container
    }

    pub fn run_command(&self, task_id: TaskId, command: &str) {
        use gtk4::prelude::*;
        use vte4::prelude::*;

        let terminal = vte4::Terminal::new();
        terminal.set_hexpand(true);
        terminal.set_vexpand(true);
        let cancellable = gtk4::gio::Cancellable::new();
        self.terminal_widgets
            .borrow_mut()
            .insert(task_id, terminal.clone());
        self.running_terminals
            .borrow_mut()
            .insert(task_id, cancellable.clone());
        self.display_selected_terminal();

        let exited_running_terminals = self.running_terminals.clone();
        let exited_sender = self.finished_sender.clone();
        terminal.connect_child_exited(move |_, exit_status| {
            let mut running_terminals = exited_running_terminals.borrow_mut();
            let should_report_finished = finish_terminal(task_id, &mut running_terminals);
            drop(running_terminals);

            if should_report_finished {
                if let Err(error) = exited_sender.send((task_id, exit_status)) {
                    eprintln!("failed to report finished task: {error}");
                }
            }
        });

        let argv = ["/bin/sh", "-lc", command];
        let failed_terminal_widgets = self.terminal_widgets.clone();
        let failed_running_terminals = self.running_terminals.clone();
        let failed_container = self.container.clone();
        let failed_selected_task_id = self.selected_task_id.clone();
        let failed_sender = self.finished_sender.clone();
        terminal.spawn_async(
            vte4::PtyFlags::DEFAULT,
            None::<&str>,
            &argv,
            &[],
            gtk4::glib::SpawnFlags::DEFAULT,
            || {},
            -1,
            Some(&cancellable),
            move |result| {
                if let Err(error) = result {
                    let mut running_terminals = failed_running_terminals.borrow_mut();
                    let transition = remove_running_terminal(task_id, &mut running_terminals);
                    drop(running_terminals);

                    let mut terminal_widgets = failed_terminal_widgets.borrow_mut();
                    let terminal_to_remove = terminal_widgets.remove(&task_id);
                    drop(terminal_widgets);

                    let removed = if let Some(terminal) = terminal_to_remove {
                        failed_container.remove(&terminal);
                        true
                    } else {
                        false
                    };
                    eprintln!("failed to start task {task_id}: {error}");
                    if transition.should_report_finished && removed {
                        if failed_selected_task_id.borrow().as_ref() == Some(&task_id) {
                            clear_container(&failed_container);
                        }
                        if let Err(send_error) = failed_sender.send((task_id, 1)) {
                            eprintln!("failed to report failed task start: {send_error}");
                        }
                    }
                }
            },
        );
    }

    pub fn cancel_command(&self, task_id: TaskId) {
        let mut running_terminals = self.running_terminals.borrow_mut();
        let transition = remove_running_terminal(task_id, &mut running_terminals);
        drop(running_terminals);

        if let Some(cancellable) = transition.terminal_to_remove {
            cancellable.cancel();
            let mut terminal_widgets = self.terminal_widgets.borrow_mut();
            let terminal_to_remove = terminal_widgets.remove(&task_id);
            drop(terminal_widgets);
            use gtk4::prelude::*;
            if let Some(terminal) = terminal_to_remove {
                self.container.remove(&terminal);
            }
        }
        self.display_selected_terminal();
    }

    pub fn select_task(&self, task_id: TaskId) {
        *self.selected_task_id.borrow_mut() = Some(task_id);
        self.display_selected_terminal();
    }

    fn display_selected_terminal(&self) {
        clear_container(&self.container);

        let Some(task_id) = *self.selected_task_id.borrow() else {
            return;
        };
        let Some(terminal) = self.terminal_widgets.borrow().get(&task_id).cloned() else {
            return;
        };

        use gtk4::prelude::*;
        self.container.append(&terminal);
    }
}

#[cfg(feature = "gui")]
fn clear_container(container: &gtk4::Box) {
    use gtk4::prelude::*;
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

#[cfg(any(feature = "gui", test))]
#[derive(Debug, PartialEq, Eq)]
struct TerminalLifecycleTransition<T> {
    should_report_finished: bool,
    terminal_to_remove: Option<T>,
}

#[cfg(any(feature = "gui", test))]
fn finish_terminal<T>(
    task_id: TaskId,
    running_terminals: &mut std::collections::HashMap<TaskId, T>,
) -> bool {
    running_terminals.remove(&task_id).is_some()
}

#[cfg(any(feature = "gui", test))]
fn remove_running_terminal<T>(
    task_id: TaskId,
    running_terminals: &mut std::collections::HashMap<TaskId, T>,
) -> TerminalLifecycleTransition<T> {
    let terminal_to_remove = running_terminals.remove(&task_id);
    let should_report_finished = terminal_to_remove.is_some();

    TerminalLifecycleTransition {
        should_report_finished,
        terminal_to_remove,
    }
}

#[cfg(feature = "gui")]
impl CommandRunner for TerminalPane {
    fn run_command(&mut self, task_id: TaskId, command: &str) {
        TerminalPane::run_command(self, task_id, command);
    }

    fn cancel_command(&mut self, task_id: TaskId) {
        TerminalPane::cancel_command(self, task_id);
    }

    fn select_task(&mut self, task_id: TaskId) {
        TerminalPane::select_task(self, task_id);
    }
}

#[cfg(feature = "gui")]
use crate::command_submission::CommandRunner;
#[cfg(any(feature = "gui", test))]
use crate::task::TaskId;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug, PartialEq, Eq)]
    struct TerminalRecord(&'static str);

    #[test]
    fn child_exit_removes_running_state_without_removing_display_state() {
        let mut running_terminals = HashMap::from([(1, TerminalRecord("running process"))]);
        let terminal_widgets = HashMap::from([(1, TerminalRecord("history remains in widget"))]);

        let should_report_finished = finish_terminal(1, &mut running_terminals);

        assert!(should_report_finished);
        assert!(!running_terminals.contains_key(&1));
        assert!(terminal_widgets.contains_key(&1));
    }

    #[test]
    fn child_exit_for_unknown_task_does_not_report_or_remove_display() {
        let mut running_terminals = HashMap::from([(2, TerminalRecord("still running"))]);

        let should_report_finished = finish_terminal(1, &mut running_terminals);

        assert!(!should_report_finished);
        assert!(running_terminals.contains_key(&2));
    }

    #[test]
    fn cancellation_returns_display_removal_target_and_clears_running_state() {
        let mut terminals = HashMap::from([(1, TerminalRecord("running widget"))]);

        let transition = remove_running_terminal(1, &mut terminals);

        assert!(transition.should_report_finished);
        assert_eq!(
            transition.terminal_to_remove,
            Some(TerminalRecord("running widget"))
        );
        assert!(!terminals.contains_key(&1));
    }
}
