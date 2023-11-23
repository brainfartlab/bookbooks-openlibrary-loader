use std::{env, process};

use formatter::{ Config, Task };

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("could not parse arguments: {err}");
        process::exit(1);
    });

    match config.task {
        Task::Assign(filename) => {
            if let Err(err) = formatter::assign(&filename) {
                println!("error extracting assignments from csv file: {err}");
                process::exit(1);
            }
        },
        Task::Format(filename) => {
            if let Err(err) = formatter::format(&filename) {
                println!("error formatting csv file: {err}");
                process::exit(1);
            }
        },
    }
}
