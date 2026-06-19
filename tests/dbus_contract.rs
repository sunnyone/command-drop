use command_drop::dbus_contract::{ADD_COMMAND_METHOD, INTERFACE_NAME, OBJECT_PATH, SERVICE_NAME};

#[test]
fn dbus_contract_values_are_stable_for_client_and_server() {
    assert_eq!(SERVICE_NAME, "dev.command_drop.CommandDrop");
    assert_eq!(OBJECT_PATH, "/dev/command_drop/CommandDrop");
    assert_eq!(INTERFACE_NAME, "dev.command_drop.CommandDrop");
    assert_eq!(ADD_COMMAND_METHOD, "AddCommand");
}

#[test]
fn object_path_uses_dbus_absolute_path_shape() {
    assert!(OBJECT_PATH.starts_with('/'));
    assert!(!OBJECT_PATH.ends_with('/'));
    assert!(!OBJECT_PATH.contains("//"));
}

#[test]
fn bus_and_interface_names_use_dot_separated_dbus_names() {
    assert!(SERVICE_NAME.contains('.'));
    assert!(INTERFACE_NAME.contains('.'));
    assert!(!SERVICE_NAME.starts_with('.'));
    assert!(!SERVICE_NAME.ends_with('.'));
    assert!(!INTERFACE_NAME.starts_with('.'));
    assert!(!INTERFACE_NAME.ends_with('.'));
}

#[test]
fn server_interface_attribute_matches_contract_constant() {
    let server_source =
        std::fs::read_to_string("src/dbus_server.rs").expect("server source should be readable");
    let expected_attribute = format!("#[zbus::interface(name = \"{INTERFACE_NAME}\")]");

    assert!(server_source.contains(&expected_attribute));
}
