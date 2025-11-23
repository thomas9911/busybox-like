use std::path::PathBuf;

enum Command {
    Copy,
    Move,
    Delete,
}

fn match_command(first_arg: &str) -> bool {
    let command = match first_arg {
        "copy" => Command::Copy,
        "move" => Command::Move,
        "delete" => Command::Delete,
        x => {
            dbg!(x);
            return true;
        }
    };

    match command {
        Command::Copy => println!("Executing copy command"),
        Command::Move => println!("Executing move command xdxd"),
        Command::Delete => println!("Executing delete command"),
    }

    false
}

fn main() -> Result<(), ()> {
    let mut args = std::env::args();
    let lets_continue = if let Some(first_arg) = args.next() {
        let pathbuf = PathBuf::from(&first_arg);
        // file_stem makes it also work on windows
        let first_arg = pathbuf.file_stem().expect("first arg is always a file");

        match_command(first_arg.to_str().expect("it came is as a string"))
    } else {
        eprintln!("no first arg?!!?");
        false
    };

    if lets_continue {
        if let Some(second_arg) = args.next() {
            match_command(&second_arg);
        } else {
            eprintln!("missing arguments");
        }
    }

    Ok(())
}
