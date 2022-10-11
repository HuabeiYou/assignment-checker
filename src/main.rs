use check::{run, Config};
use clap::{arg, Command};
use std::process;
mod arguments;

fn cli() -> Command {
    Command::new("check")
        .about(arguments::NAME)
        .subcommand_required(false)
        .allow_external_subcommands(false)
        .arg_required_else_help(true)
        .arg(
            arg!(<FILES> ... "Files to test, need to be in the same directory with the checker.")
                .num_args(1..)
                .required(true),
        )
        .arg(
            arg!(-p --phone <PHONE> "Registered phone number")
                .num_args(1)
                .required(true),
        )
}

fn main() {
    let matches = cli().get_matches();
    let phone = matches
        .get_one::<String>("phone")
        .expect("phone is required.");
    let files = matches
        .get_many::<String>("FILES")
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    let config = match Config::build(arguments::LESSON, phone, files) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };
    if let Err(e) = run(config) {
        eprintln!("{e}");
        process::exit(1);
    }
}
