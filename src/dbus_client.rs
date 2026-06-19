use std::error::Error;

use zbus::blocking::{Connection, Proxy};

use crate::dbus_contract::{ADD_COMMAND_METHOD, INTERFACE_NAME, OBJECT_PATH, SERVICE_NAME};

pub fn send_command(command: &str) -> Result<(), Box<dyn Error>> {
    let connection = Connection::session()?;
    let proxy = Proxy::new(&connection, SERVICE_NAME, OBJECT_PATH, INTERFACE_NAME)?;
    proxy.call::<_, _, ()>(ADD_COMMAND_METHOD, &(command))?;
    Ok(())
}
