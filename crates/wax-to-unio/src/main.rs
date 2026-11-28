//! `wax-to-unio FILE.wax`: prints the Unio program that means the same as a Wax script, for running on Ralph
//! (D19).
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

use std::process::ExitCode;

use wax_to_unio::{Untranslatable, test_host_doors, translate};

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let [path] = arguments.as_slice() else {
        eprintln!("usage: wax-to-unio FILE.wax");
        return ExitCode::from(6);
    };
    let Ok(source) = std::fs::read_to_string(path) else {
        eprintln!("wax-to-unio: `{path}` can't be read");
        return ExitCode::from(2);
    };
    match translate(&source, &test_host_doors()) {
        Ok(unio) => {
            print!("{unio}");
            ExitCode::SUCCESS
        }
        Err(Untranslatable::Refused(messages)) => {
            for mut message in messages {
                message.locate(path, &source);
                eprint!("{message}");
            }
            ExitCode::from(1)
        }
        Err(Untranslatable::NotShared(what)) => {
            eprintln!("wax-to-unio: `{path}` uses {what}, which Wax doesn't share with Unio");
            ExitCode::from(3)
        }
    }
}
