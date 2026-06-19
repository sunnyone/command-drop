# Command Drop

## What is this?

This is a tool that prevents your workspace from becoming cluttered with windows when you want to open commands in separate terminals. Once you start it, you can send commands to it from the outside while keeping the number of terminal windows limited.

Command Drop runs a single host process and accepts command submissions over the user session DBus. The default build starts a GTK/VTE UI, shows submitted tasks, and runs commands in embedded terminal panes.

## Requirements

- Rust stable toolchain with Cargo
- A Linux desktop/session environment with DBus
- GTK 4 and VTE 4 development libraries for the default GUI build

On Debian/Ubuntu-like systems, the native packages are typically:

```sh
sudo apt install libgtk-4-dev libvte-2.91-gtk4-dev dbus
```

Package names vary by distribution.

## Build

Build the default GUI binary:

```sh
cargo build
```

## Usage

### 1. Start the UI

Start the UI:

```sh
command-drop host
```

During development, you can run the host without installing the binary:

```sh
cargo run -- host
```

### 2. Add a command

Submit a command to the running host from another shell:

```sh
command-drop run "sleep 10 && echo done"
```

With Cargo:

```sh
cargo run -- run "sleep 10 && echo done"
```

The command string is sent as one argument, so quote shell pipelines, redirects, and compound commands.

### 3. Show help

```sh
command-drop help
```

## Development

Run the test suite:

```sh
cargo test
```

DBus integration tests use `dbus-run-session`, so that command must be available in the test environment.
