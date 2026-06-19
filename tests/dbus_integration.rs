use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use command_drop::app::{send_command_to_host, CommandHost};
use command_drop::command_submission::CommandRunner;
use command_drop::task::TaskId;

static DBUS_TEST_LOCK: Mutex<()> = Mutex::new(());
const CHILD_ENV: &str = "TAKT_DBUS_INTEGRATION_CHILD";
const CHILD_CASE_ENV: &str = "TAKT_DBUS_INTEGRATION_CASE";
const CASE_SEND_COMMAND_REACHES_RUNNER: &str = "send_command_to_host_reaches_production_runner";
const CASE_ACCEPTS_VALID_AFTER_BLANK: &str = "host_accepts_valid_command_after_blank_command_error";
const CASE_RUNS_MULTIPLE_WITHIN_LIMIT: &str = "host_runs_multiple_commands_in_order_within_limit";
const CHILD_TIMEOUT: Duration = Duration::from_secs(2);
const CHILD_PROCESS_TIMEOUT: Duration = Duration::from_secs(5);

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

    fn wait_for_started_commands(&self, expected_len: usize) -> Vec<(TaskId, String)> {
        let deadline = std::time::Instant::now() + CHILD_TIMEOUT;

        while std::time::Instant::now() < deadline {
            let started = self.started_commands();
            if started.len() >= expected_len {
                return started;
            }

            std::thread::sleep(Duration::from_millis(10));
        }

        self.started_commands()
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
fn dbus_send_command_reaches_production_runner_path() {
    run_child_test_in_isolated_dbus(CASE_SEND_COMMAND_REACHES_RUNNER);
}

#[test]
fn dbus_blank_command_error_does_not_stop_next_valid_command() {
    run_child_test_in_isolated_dbus(CASE_ACCEPTS_VALID_AFTER_BLANK);
}

#[test]
fn dbus_runs_multiple_commands_in_order_within_limit() {
    run_child_test_in_isolated_dbus(CASE_RUNS_MULTIPLE_WITHIN_LIMIT);
}

#[test]
fn dbus_child_run_requested_case() {
    if std::env::var_os(CHILD_ENV).is_none() {
        return;
    }

    match std::env::var(CHILD_CASE_ENV)
        .expect("DBus child test requires an explicit case name")
        .as_str()
    {
        CASE_SEND_COMMAND_REACHES_RUNNER => send_command_to_host_reaches_production_runner(),
        CASE_ACCEPTS_VALID_AFTER_BLANK => host_accepts_valid_command_after_blank_command_error(),
        CASE_RUNS_MULTIPLE_WITHIN_LIMIT => host_runs_multiple_commands_in_order_within_limit(),
        unknown => panic!("unknown DBus integration case: {unknown}"),
    }
}

fn send_command_to_host_reaches_production_runner() {
    let _guard = lock_dbus_test();
    let runner = RecordingRunner::default();
    let _host =
        CommandHost::start(1, runner.clone()).expect("host should start on isolated DBus session");

    send_command_to_host("echo from dbus").expect("DBus client should send command");

    assert_eq!(
        runner.wait_for_started_commands(1),
        vec![(0, "echo from dbus".to_string())]
    );
}

fn host_runs_multiple_commands_in_order_within_limit() {
    let _guard = lock_dbus_test();
    let runner = RecordingRunner::default();
    let _host =
        CommandHost::start(2, runner.clone()).expect("host should start on isolated DBus session");

    send_command_to_host("echo first").expect("first DBus command should be sent");
    send_command_to_host("echo second").expect("second DBus command should be sent");

    assert_eq!(
        runner.wait_for_started_commands(2),
        vec![
            (0, "echo first".to_string()),
            (1, "echo second".to_string())
        ]
    );
}

fn host_accepts_valid_command_after_blank_command_error() {
    let _guard = lock_dbus_test();
    let runner = RecordingRunner::default();
    let _host =
        CommandHost::start(1, runner.clone()).expect("host should start on isolated DBus session");

    let blank = send_command_to_host(" \t\n ");
    assert!(blank.is_err());
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(runner.started_commands(), Vec::new());

    send_command_to_host("echo after invalid args").expect("host should keep serving after error");

    assert_eq!(
        runner.wait_for_started_commands(1),
        vec![(0, "echo after invalid args".to_string())]
    );
}

fn run_child_test_in_isolated_dbus(test_name: &str) {
    let _guard = lock_dbus_test();
    let test_binary = std::env::current_exe().expect("current test binary should be available");
    let child = Command::new("dbus-run-session")
        .arg("--")
        .arg(test_binary)
        .arg("--exact")
        .arg("dbus_child_run_requested_case")
        .arg("--nocapture")
        .env(CHILD_ENV, "1")
        .env(CHILD_CASE_ENV, test_name)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("dbus-run-session should be installed for DBus integration tests");

    let status = wait_for_child_test(child, test_name);
    assert!(status.success());
}

fn lock_dbus_test() -> MutexGuard<'static, ()> {
    DBUS_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn wait_for_child_test(mut child: Child, test_name: &str) -> ExitStatus {
    let deadline = std::time::Instant::now() + CHILD_PROCESS_TIMEOUT;

    loop {
        if let Some(status) = child
            .try_wait()
            .expect("DBus integration child status should be readable")
        {
            return status;
        }

        if std::time::Instant::now() >= deadline {
            child
                .kill()
                .expect("timed out DBus integration child should be killable");
            child
                .wait()
                .expect("killed DBus integration child should be waitable");
            panic!("DBus integration child timed out for case: {test_name}");
        }

        std::thread::sleep(Duration::from_millis(10));
    }
}
