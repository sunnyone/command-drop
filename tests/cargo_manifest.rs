use std::fs;
use std::path::Path;

#[test]
fn package_name_declares_command_drop_binary_contract() {
    let manifest = fs::read_to_string("Cargo.toml").expect("Cargo.toml should be readable");

    assert!(manifest.contains("name = \"command-drop\""));
}

#[test]
fn package_manifest_does_not_declare_legacy_package_or_binary_name() {
    let manifest = fs::read_to_string("Cargo.toml").expect("Cargo.toml should be readable");
    let legacy_name = ["takt", "test"].concat();
    let legacy_name_line = format!("name = \"{legacy_name}\"");

    assert!(!manifest.contains(&legacy_name_line));
}

#[test]
fn gui_feature_declares_gtk_and_vte_dependencies() {
    let manifest = fs::read_to_string("Cargo.toml").expect("Cargo.toml should be readable");

    assert!(manifest.contains("gui = [\"dep:gtk4\", \"dep:vte4\"]"));
    assert!(manifest.contains("gtk4 = { version = \"0.11\", optional = true }"));
    assert!(manifest.contains("vte4 = { version = \"0.10\", optional = true }"));
}

#[test]
fn dbus_infrastructure_modules_are_not_public_api() {
    let lib = fs::read_to_string("src/lib.rs").expect("src/lib.rs should be readable");

    assert!(lib.contains("mod dbus_client;"));
    assert!(lib.contains("mod dbus_server;"));
    assert!(!lib.contains("pub mod dbus_client;"));
    assert!(!lib.contains("pub mod dbus_server;"));
}

#[test]
fn gui_and_terminal_modules_exist_for_host_feature() {
    let app = fs::read_to_string("src/app.rs").expect("src/app.rs should be readable");
    let terminal =
        fs::read_to_string("src/terminal.rs").expect("src/terminal.rs should be readable");

    assert!(app.contains("ui::run_host()"));
    assert!(terminal.contains("vte4::Terminal"));
    assert!(terminal.contains("spawn_async"));
}

#[test]
fn default_build_enables_gui_host_feature() {
    let manifest = fs::read_to_string("Cargo.toml").expect("Cargo.toml should be readable");

    assert!(manifest.contains("default = [\"gui\"]"));
}

#[test]
fn repository_root_does_not_contain_unrelated_html_artifacts() {
    assert!(
        !Path::new("index.html").exists(),
        "unrelated root HTML artifacts must not be included in this Rust/GTK UI scope"
    );
}
