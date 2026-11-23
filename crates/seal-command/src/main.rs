//! The `seal` command: checks a script, and runs it with the test host (D47, tests/README.md).
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let (Some(command), Some(path)) = (arguments.first(), arguments.get(1)) else {
        eprintln!("usage: seal run FILE.seal, or seal check FILE.seal");
        return ExitCode::from(6);
    };
    let Ok(source) = std::fs::read_to_string(path) else {
        eprintln!("seal: `{path}` can't be read");
        return ExitCode::from(2);
    };
    if let Err(mut message) = seal::syntax::parser::parse(&source) {
        message.locate(path, &source);
        eprint!("{message}");
        return ExitCode::from(1);
    }
    eprintln!("NotBuiltYet: `seal {command}` isn't built yet");
    ExitCode::from(3)
}
