use std::error::Error;
use std::sync::mpsc;

#[cfg(feature = "gui")]
mod controls;
#[cfg(feature = "gui")]
mod host_state;
#[cfg(feature = "gui")]
mod task_list;
#[cfg(feature = "gui")]
mod window;

#[cfg(not(feature = "gui"))]
use crate::dbus_server::{process_received_commands, CommandReceiver, RunningCommandServer};
#[cfg(feature = "gui")]
use crate::dbus_server::{CommandReceiver, RunningCommandServer};

#[cfg(feature = "gui")]
const GUI_APPLICATION_ID: &str = "dev.command_drop.CommandDrop.HostUi";
#[cfg(feature = "gui")]
const GUI_APPLICATION_ARGS: [&str; 1] = ["command-drop"];

#[cfg(feature = "gui")]
pub fn run_host() -> Result<(), Box<dyn Error>> {
    use gtk4::prelude::*;

    let (sender, receiver) = mpsc::channel();
    let server = RunningCommandServer::start(CommandReceiver::new(sender))?;
    let app = gtk4::Application::builder()
        .application_id(GUI_APPLICATION_ID)
        .build();
    let server = std::rc::Rc::new(server);
    app.connect_shutdown(move |_| {
        let _server = server.clone();
    });

    let receiver = std::rc::Rc::new(std::cell::RefCell::new(Some(receiver)));

    app.connect_activate(move |app| {
        let receiver = receiver
            .borrow_mut()
            .take()
            .expect("application activation should only initialize host once");
        window::build_window(app, receiver);
    });

    app.run_with_args(&GUI_APPLICATION_ARGS);
    Ok(())
}

#[cfg(not(feature = "gui"))]
pub fn run_host() -> Result<(), Box<dyn Error>> {
    let (sender, receiver) = mpsc::channel();
    let _server = RunningCommandServer::start(CommandReceiver::new(sender))?;
    process_received_commands(receiver)
}
