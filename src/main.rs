use std::env;
use std::error::Error;

use command_drop::app::{run_host, send_command_to_host};
use command_drop::cli::{format_help, parse_args, CliCommand};

fn main() -> Result<(), Box<dyn Error>> {
    match parse_args(env::args())? {
        CliCommand::Host => run_host(),
        CliCommand::Run { command } => send_command_to_host(&command),
        CliCommand::Help => {
            print!("{}", format_help("command-drop"));
            Ok(())
        }
    }
}
