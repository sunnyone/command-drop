use command_drop::cli::{format_help, parse_args, CliCommand};

use std::process::Command;

#[test]
fn parses_host_subcommand() {
    let args = ["command-drop", "host"];

    let command = parse_args(args).expect("host subcommand should parse");

    assert_eq!(command, CliCommand::Host);
}

#[test]
fn parses_run_subcommand_with_command_string() {
    let args = ["command-drop", "run", "echo hello && pwd"];

    let command = parse_args(args).expect("run subcommand should parse");

    assert_eq!(
        command,
        CliCommand::Run {
            command: "echo hello && pwd".to_string()
        }
    );
}

#[test]
fn rejects_run_subcommand_without_command_string() {
    let args = ["command-drop", "run"];

    let command = parse_args(args);

    assert!(command.is_err());
}

#[test]
fn rejects_run_subcommand_with_whitespace_only_command() {
    let args = ["command-drop", "run", " \t\n "];

    let command = parse_args(args);

    assert!(command.is_err());
}

#[test]
fn parses_help_subcommand() {
    let args = ["command-drop", "help"];

    let command = parse_args(args).expect("help subcommand should parse");

    assert_eq!(command, CliCommand::Help);
}

#[test]
fn parses_long_help_flag_as_top_level_help() {
    let args = ["command-drop", "--help"];

    let command = parse_args(args).expect("long help flag should parse");

    assert_eq!(command, CliCommand::Help);
}

#[test]
fn parses_short_help_flag_as_top_level_help() {
    let args = ["command-drop", "-h"];

    let command = parse_args(args).expect("short help flag should parse");

    assert_eq!(command, CliCommand::Help);
}

#[test]
fn help_text_lists_available_subcommands() {
    let help = format_help("command-drop");

    assert!(help.contains("Subcommands:"));
    assert!(help.contains("host"));
    assert!(help.contains("run"));
    assert!(help.contains("help"));
}

#[test]
fn help_text_uses_given_program_name_in_usage() {
    let help = format_help("cargo run --");

    assert!(help.contains("Usage: cargo run -- <SUBCOMMAND>"));
}

#[test]
fn help_subcommand_prints_available_subcommands_from_binary_entrypoint() {
    let output = Command::new(env!("CARGO_BIN_EXE_command-drop"))
        .arg("help")
        .output()
        .expect("help subcommand should execute");

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).expect("help output should be utf-8");
    assert!(stdout.contains("Subcommands:"));
    assert!(stdout.contains("host"));
    assert!(stdout.contains("run"));
    assert!(stdout.contains("help"));
}
