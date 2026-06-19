use std::fs;
use std::path::Path;

use command_drop::dbus_contract::SERVICE_NAME;

#[test]
fn gtk_application_id_is_separate_from_dbus_service_name() {
    let sources = read_rust_sources(Path::new("src"));
    let gui_application_id = find_string_const(&sources, "GUI_APPLICATION_ID")
        .expect("GUI application id should be defined as an explicit host UI constant");

    assert_eq!(gui_application_id, "dev.command_drop.CommandDrop.HostUi");
    assert_ne!(gui_application_id, SERVICE_NAME);
    assert!(sources.contains("application_id(GUI_APPLICATION_ID)"));
    assert!(sources.contains("run_with_args(&GUI_APPLICATION_ARGS)"));
    assert!(!sources.contains("application_id(SERVICE_NAME)"));
}

#[test]
fn gtk_application_uses_command_drop_binary_name_for_process_args() {
    let sources = read_rust_sources(Path::new("src"));
    let gui_application_args = find_string_array_const(&sources, "GUI_APPLICATION_ARGS")
        .expect("GUI application args should be defined as an explicit host UI constant");

    assert_eq!(gui_application_args, vec!["command-drop"]);
}

#[test]
fn host_window_title_uses_command_drop_display_name() {
    let window_source =
        fs::read_to_string("src/ui/window.rs").expect("window source should be readable");

    assert!(window_source.contains(".title(\"Command Drop\")"));
}

#[test]
fn initial_pane_position_uses_three_to_seven_split() {
    let window_source =
        fs::read_to_string("src/ui/window.rs").expect("window source should be readable");

    assert!(window_source.contains("const DEFAULT_WINDOW_WIDTH: i32 = 1100;"));
    assert!(window_source
        .contains("const INITIAL_LEFT_PANE_WIDTH: i32 = DEFAULT_WINDOW_WIDTH * 3 / 10;"));
    assert!(window_source.contains("panes.set_position(INITIAL_LEFT_PANE_WIDTH);"));
}

#[test]
fn terminal_runner_contract_supports_explicit_selection() {
    let command_submission_source = fs::read_to_string("src/command_submission.rs")
        .expect("command submission source should be readable");
    let terminal_source =
        fs::read_to_string("src/terminal.rs").expect("terminal source should be readable");

    assert!(command_submission_source.contains("fn select_task(&mut self, task_id: TaskId);"));
    assert!(command_submission_source.contains("pub fn select_task("));
    assert!(terminal_source.contains("pub fn select_task(&self, task_id: TaskId)"));
    assert!(terminal_source.contains("TerminalPane::select_task(self, task_id);"));
}

#[test]
fn terminal_run_command_does_not_append_each_started_terminal_to_right_pane() {
    let terminal_source =
        fs::read_to_string("src/terminal.rs").expect("terminal source should be readable");
    let run_command_body = rust_function_body(&terminal_source, "run_command")
        .expect("terminal run_command body should be readable");

    assert!(!run_command_body.contains("self.container.append(&terminal);"));
}

#[test]
fn list_row_selection_is_wired_to_existing_task_selection() {
    let window_source =
        fs::read_to_string("src/ui/window.rs").expect("window source should be readable");
    let selection_body = rust_function_body(&window_source, "connect_task_selection")
        .expect("selection handler body should be readable");

    assert!(selection_body.contains("list.connect_row_selected"));
    assert!(selection_body.contains("is_syncing_selection()"));
    assert!(selection_body.contains("task_id_for_row(row)"));
    assert!(selection_body.contains("controller.borrow_mut().select_task(task_id)"));
    assert!(!selection_body.contains("add_command("));
}

#[test]
fn selected_terminal_display_clears_container_before_appending_one_terminal() {
    let terminal_source =
        fs::read_to_string("src/terminal.rs").expect("terminal source should be readable");
    let display_body = rust_function_body(&terminal_source, "display_selected_terminal")
        .expect("terminal display body should be readable");

    let clear_position = display_body
        .find("clear_container(&self.container);")
        .expect("display should clear the right pane before appending");
    let append_position = display_body
        .find("self.container.append(&terminal);")
        .expect("display should append the selected terminal");

    assert!(clear_position < append_position);
    assert_eq!(display_body.matches("self.container.append(").count(), 1);
    assert!(display_body.contains("self.terminal_widgets.borrow().get(&task_id)"));
}

#[test]
fn command_entry_activate_submits_through_same_path_as_add_button() {
    let controls_source =
        fs::read_to_string("src/ui/controls.rs").expect("controls source should be readable");
    let add_button_body = rust_function_body(&controls_source, "connect_add_button")
        .expect("add button handler body should be readable");
    let activate_body = rust_function_body(&controls_source, "connect_command_entry")
        .expect("command entry activate handler body should be readable");
    let submit_body = rust_function_body(&controls_source, "submit_command_entry")
        .expect("shared command submission body should be readable");

    assert!(add_button_body.contains("submit_command_entry("));
    assert!(activate_body.contains("command_entry.connect_activate"));
    assert!(activate_body.contains("submit_command_entry("));
    assert!(submit_body.contains("controller.borrow_mut().add_command(&command)"));
    assert!(submit_body.contains("sync_outcome_tasks(&controller, &task_list, outcome)"));
    assert!(submit_body.contains("command_entry.set_text(\"\")"));
    assert!(submit_body.contains("eprintln!(\"rejected command: {error}\")"));
}

#[test]
fn command_entry_submission_does_not_duplicate_add_command_calls() {
    let controls_source =
        fs::read_to_string("src/ui/controls.rs").expect("controls source should be readable");

    assert_eq!(
        controls_source.matches(".add_command(&command)").count(),
        1,
        "button click and Enter activation must share the same Add implementation"
    );
}

#[test]
fn window_and_paned_allow_content_to_expand_with_the_window() {
    let window_source =
        fs::read_to_string("src/ui/window.rs").expect("window source should be readable");
    let build_window_body = rust_function_body(&window_source, "build_window")
        .expect("window build body should be readable");
    let build_panes_body = rust_function_body(&window_source, "build_panes")
        .expect("pane build body should be readable");

    assert!(build_window_body.contains("root.set_hexpand(true);"));
    assert!(build_window_body.contains("root.set_vexpand(true);"));
    assert!(build_panes_body.contains("panes.set_hexpand(true);"));
    assert!(build_panes_body.contains("panes.set_vexpand(true);"));
    assert!(build_panes_body.contains("panes.set_resize_start_child(true);"));
    assert!(build_panes_body.contains("panes.set_resize_end_child(true);"));
    assert!(build_panes_body.contains("panes.set_shrink_start_child(true);"));
    assert!(build_panes_body.contains("panes.set_shrink_end_child(true);"));
}

#[test]
fn task_list_exposes_scrollable_pane_widget_and_list_selection_target() {
    let task_list_source =
        fs::read_to_string("src/ui/task_list.rs").expect("task list source should be readable");
    let new_body = rust_function_body(&task_list_source, "new")
        .expect("task list new body should be readable");

    assert!(task_list_source.contains("scroller: gtk4::ScrolledWindow"));
    assert!(task_list_source.contains("pub(crate) fn widget(&self) -> &gtk4::ScrolledWindow"));
    assert!(task_list_source.contains("pub(crate) fn list(&self) -> &gtk4::ListBox"));
    assert!(new_body.contains("list.set_hexpand(true);"));
    assert!(new_body.contains("list.set_vexpand(true);"));
    assert!(new_body.contains("scroller.set_hexpand(true);"));
    assert!(new_body.contains("scroller.set_vexpand(true);"));
    assert!(new_body.contains("scroller.set_child(Some(&list));"));
}

#[test]
fn task_selection_is_connected_to_inner_list_not_scroll_container() {
    let window_source =
        fs::read_to_string("src/ui/window.rs").expect("window source should be readable");
    let selection_body = rust_function_body(&window_source, "connect_task_selection")
        .expect("selection handler body should be readable");

    assert!(selection_body.contains("task_list.borrow().list().clone()"));
    assert!(selection_body.contains("list.connect_row_selected"));
}

#[test]
fn terminal_pane_and_terminal_widgets_expand_to_fill_the_right_pane() {
    let terminal_source =
        fs::read_to_string("src/terminal.rs").expect("terminal source should be readable");
    let new_body =
        rust_function_body(&terminal_source, "new").expect("terminal new body should be readable");
    let run_command_body = rust_function_body(&terminal_source, "run_command")
        .expect("terminal run command body should be readable");

    assert!(new_body.contains("container.set_hexpand(true);"));
    assert!(new_body.contains("container.set_vexpand(true);"));
    assert!(run_command_body.contains("terminal.set_hexpand(true);"));
    assert!(run_command_body.contains("terminal.set_vexpand(true);"));
}

fn read_rust_sources(root: &Path) -> String {
    let mut sources = String::new();
    for entry in fs::read_dir(root).expect("source directory should be readable") {
        let path = entry.expect("source entry should be readable").path();
        if path.is_dir() {
            sources.push_str(&read_rust_sources(&path));
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            sources
                .push_str(&fs::read_to_string(&path).expect("Rust source file should be readable"));
        }
    }
    sources
}

fn rust_function_body(source: &str, function_name: &str) -> Option<String> {
    let signature_start = source
        .find(&format!("pub fn {function_name}("))
        .or_else(|| source.find(&format!("fn {function_name}(")))?;
    let body_start = source[signature_start..].find('{')? + signature_start;
    let mut depth = 0_usize;

    for (offset, character) in source[body_start..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(source[body_start..=body_start + offset].to_string());
                }
            }
            _ => {}
        }
    }

    None
}

fn find_string_const(sources: &str, name: &str) -> Option<String> {
    let prefix = format!("const {name}: &str = \"");
    let start = sources.find(&prefix)? + prefix.len();
    let value = sources[start..].split('"').next()?;

    Some(value.to_string())
}

fn find_string_array_const(sources: &str, name: &str) -> Option<Vec<String>> {
    let prefix = format!("const {name}: [&str; 1] = [");
    let start = sources.find(&prefix)? + prefix.len();
    let value = sources[start..].split("];").next()?;

    Some(
        value
            .split(',')
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .map(|part| part.trim_matches('"').to_string())
            .collect(),
    )
}
