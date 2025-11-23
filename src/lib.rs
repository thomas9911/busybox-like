#![no_std]

/// Core command enum and parsing logic.
/// This crate root is `no_std` and doesn't perform any I/O.
pub enum Command {
    Copy,
    Move,
    Delete,
}

/// Parse a command from a string. Returns Some(Command) when recognized,
/// or None for unknown commands. No allocation, uses only `core`.
pub fn parse_command(first_arg: &str) -> Option<Command> {
    match first_arg {
        "copy" => Some(Command::Copy),
        "move" => Some(Command::Move),
        "delete" => Some(Command::Delete),
        _ => None,
    }
}

/// Get a human readable message for a command. Keeps string literals so
/// the function remains allocation-free and `no_std` friendly.
pub fn message_for(cmd: &Command) -> &'static str {
    match cmd {
        Command::Copy => "Executing copy command",
        Command::Move => "Executing move command xdxd",
        Command::Delete => "Executing delete command",
    }
}
