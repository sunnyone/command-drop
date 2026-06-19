use std::error::Error;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::command_submission::{CommandRunner, HostCommandController};
use crate::dbus_client;
use crate::dbus_server::{CommandReceiver, RunningCommandServer};
use crate::ui;

pub struct CommandHost {
    server: Option<RunningCommandServer>,
    stop_sender: Option<mpsc::Sender<()>>,
    worker: Option<thread::JoinHandle<()>>,
}

impl CommandHost {
    pub fn start<R>(concurrency_limit: usize, runner: R) -> Result<Self, Box<dyn Error>>
    where
        R: CommandRunner + Send + 'static,
    {
        let (sender, receiver) = mpsc::channel::<String>();
        let (stop_sender, stop_receiver) = mpsc::channel::<()>();
        let mut controller = HostCommandController::new(concurrency_limit, runner)?;
        let worker = thread::spawn(move || loop {
            if stop_receiver.try_recv().is_ok() {
                break;
            }

            match receiver.recv_timeout(Duration::from_millis(50)) {
                Ok(command) => {
                    if let Err(error) = controller.add_command(&command) {
                        eprintln!("rejected command: {error}");
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        });

        Ok(Self {
            server: Some(RunningCommandServer::start(CommandReceiver::new(sender))?),
            stop_sender: Some(stop_sender),
            worker: Some(worker),
        })
    }
}

impl Drop for CommandHost {
    fn drop(&mut self) {
        drop(self.server.take());
        if let Some(stop_sender) = self.stop_sender.take() {
            let _ = stop_sender.send(());
        }
        if let Some(worker) = self.worker.take() {
            if let Err(error) = worker.join() {
                eprintln!("failed to stop command host worker: {error:?}");
            }
        }
    }
}

pub fn run_host() -> Result<(), Box<dyn Error>> {
    ui::run_host()
}

pub fn send_command_to_host(command: &str) -> Result<(), Box<dyn Error>> {
    dbus_client::send_command(command)
}
