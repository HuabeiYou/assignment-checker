use check::{run, send_analytic, Config};
use clap::{arg, Command};
use std::process;
mod constants;

fn cli() -> Command {
    Command::new("check")
        .about(constants::BANNER)
        .subcommand_required(false)
        .allow_external_subcommands(false)
        .arg_required_else_help(true)
        .arg(
            arg!(<FILES> ... "Files to test, need to be in the same directory with the checker.")
                .num_args(1..)
                .required(true),
        )
        .arg(
            arg!(-p --phone <PHONE> "Registered phone number.")
                .num_args(1)
                .required(true),
        )
        .arg(
            arg!(-id --test_id <TEST_ID> "Test set id, you can get it from the instructor.")
                .num_args(1)
                .required(true),
        )
}

fn main() {
    let matches = cli().get_matches();
    let phone = matches
        .get_one::<&str>("phone")
        .expect("phone is required.");
    let test_set = matches
        .get_one::<&str>("test_id")
        .expect("test id is required.");
    let files = matches
        .get_many::<&str>("FILES")
        .expect("At least one file is required.")
        .copied()
        .collect::<Vec<_>>();

    let config = match Config::build(test_set, phone, files) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };
    let data = match run(config) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };
    println!("{}", data.result);
    if send_analytic(data).is_err() {
        // do nothing
        process::exit(1);
    };
}
