use std::error::Error;
#[cfg(not(feature = "gui"))]
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

use zbus::blocking::{connection::Builder, Connection};

#[cfg(not(feature = "gui"))]
use crate::command_submission::submit_command;
use crate::dbus_contract::{OBJECT_PATH, SERVICE_NAME};
#[cfg(not(feature = "gui"))]
use crate::task::TaskManager;

pub struct CommandReceiver {
    sender: Sender<String>,
}

impl CommandReceiver {
    pub fn new(sender: Sender<String>) -> Self {
        Self { sender }
    }
}

#[zbus::interface(name = "dev.command_drop.CommandDrop")]
impl CommandReceiver {
    fn add_command(&self, command: &str) -> zbus::fdo::Result<()> {
        if command.trim().is_empty() {
            return Err(zbus::fdo::Error::InvalidArgs(
                "command must not be empty".to_string(),
            ));
        }

        self.sender
            .send(command.to_string())
            .map_err(|error| zbus::fdo::Error::Failed(format!("failed to queue command: {error}")))
    }
}

pub struct RunningCommandServer {
    _connection: Connection,
}

impl RunningCommandServer {
    pub fn start(receiver: CommandReceiver) -> Result<Self, Box<dyn Error>> {
        let connection = Builder::session()?
            .serve_at(OBJECT_PATH, receiver)?
            .name(SERVICE_NAME)?
            .build()?;
        Ok(Self {
            _connection: connection,
        })
    }
}

#[cfg(not(feature = "gui"))]
pub fn submit_received_command(
    manager: &mut TaskManager,
    command: &str,
) -> Result<crate::task::TaskOutcome, crate::task::TaskError> {
    submit_command(manager, command)
}

#[cfg(not(feature = "gui"))]
pub fn process_received_commands(receiver: Receiver<String>) -> Result<(), Box<dyn Error>> {
    let mut manager = TaskManager::new(1)?;

    for command in receiver {
        match submit_received_command(&mut manager, &command) {
            Ok(outcome) => {
                if let Some(task_id) = outcome.task_id {
                    println!("queued task {task_id}");
                }
            }
            Err(error) => eprintln!("rejected command: {error}"),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use super::CommandReceiver;

    #[test]
    fn command_receiver_rejects_blank_command_before_queueing() {
        let (sender, receiver) = mpsc::channel();
        let command_receiver = CommandReceiver::new(sender);

        let result = command_receiver.add_command(" \t\n ");

        assert!(result.is_err());
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn command_receiver_queues_non_blank_command() {
        let (sender, receiver) = mpsc::channel();
        let command_receiver = CommandReceiver::new(sender);

        command_receiver
            .add_command("echo from dbus")
            .expect("non-empty command should be queued");

        assert_eq!(
            receiver.try_recv().expect("command should be queued"),
            "echo from dbus"
        );
    }
}
