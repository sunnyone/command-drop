use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliCommand {
    Host,
    Run { command: String },
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    MissingSubcommand,
    UnknownSubcommand(String),
    MissingRunCommand,
    EmptyRunCommand,
    UnexpectedArgument(String),
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSubcommand => write!(f, "subcommand is required"),
            Self::UnknownSubcommand(command) => write!(f, "unknown subcommand: {command}"),
            Self::MissingRunCommand => write!(f, "run command requires a command string"),
            Self::EmptyRunCommand => write!(f, "run command must not be empty"),
            Self::UnexpectedArgument(argument) => write!(f, "unexpected argument: {argument}"),
        }
    }
}

impl Error for CliError {}

struct Subcommand {
    name: &'static str,
    description: &'static str,
}

const HOST_SUBCOMMAND: &str = "host";
const RUN_SUBCOMMAND: &str = "run";
const HELP_SUBCOMMAND: &str = "help";
const HELP_FLAGS: [&str; 2] = ["--help", "-h"];
const SUBCOMMANDS: [Subcommand; 3] = [
    Subcommand {
        name: HOST_SUBCOMMAND,
        description: "Start the GUI command host",
    },
    Subcommand {
        name: RUN_SUBCOMMAND,
        description: "Send a command string to the running host",
    },
    Subcommand {
        name: HELP_SUBCOMMAND,
        description: "Show available subcommands",
    },
];

pub fn parse_args<I, S>(args: I) -> Result<CliCommand, CliError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut values = args.into_iter();
    let _program_name = values.next();
    let subcommand = values.next().ok_or(CliError::MissingSubcommand)?;

    match subcommand.as_ref() {
        HOST_SUBCOMMAND => parse_host(values),
        RUN_SUBCOMMAND => parse_run(values),
        HELP_SUBCOMMAND => parse_help(values),
        flag if HELP_FLAGS.contains(&flag) => parse_help(values),
        unknown => Err(CliError::UnknownSubcommand(unknown.to_string())),
    }
}

pub fn format_help(program_name: &str) -> String {
    let mut help = format!("Usage: {program_name} <SUBCOMMAND>\n\nSubcommands:\n");
    for subcommand in SUBCOMMANDS {
        help.push_str(&format!(
            "  {:<8} {}\n",
            subcommand.name, subcommand.description
        ));
    }
    help
}

fn parse_host<I, S>(mut values: I) -> Result<CliCommand, CliError>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    if let Some(argument) = values.next() {
        return Err(CliError::UnexpectedArgument(argument.as_ref().to_string()));
    }

    Ok(CliCommand::Host)
}

fn parse_run<I, S>(mut values: I) -> Result<CliCommand, CliError>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    let command = values.next().ok_or(CliError::MissingRunCommand)?;
    if let Some(argument) = values.next() {
        return Err(CliError::UnexpectedArgument(argument.as_ref().to_string()));
    }

    let command = command.as_ref();
    if command.trim().is_empty() {
        return Err(CliError::EmptyRunCommand);
    }

    Ok(CliCommand::Run {
        command: command.to_string(),
    })
}

fn parse_help<I, S>(mut values: I) -> Result<CliCommand, CliError>
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    if let Some(argument) = values.next() {
        return Err(CliError::UnexpectedArgument(argument.as_ref().to_string()));
    }

    Ok(CliCommand::Help)
}
