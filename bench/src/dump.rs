//! `seal-dump FILE.seal`: prints the bytecode a script compiles to, for whoever works on the VM.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: seal-dump FILE.seal");
        return ExitCode::from(6);
    };
    let Ok(source) = std::fs::read_to_string(&path) else {
        eprintln!("`{path}` can't be read");
        return ExitCode::from(2);
    };
    let compiled = seal::syntax::parser::parse(&source)
        .map_err(|message| vec![message])
        .and_then(|tree| seal::check::check(&tree, &[]))
        .and_then(|program| seal::compile::compiler::compile(&program, &[]).map_err(|message| vec![message]));
    match compiled {
        Ok(compiled) => {
            for (index, (instruction, place)) in compiled.code.iter().zip(&compiled.places).enumerate() {
                if let Some(procedure) = compiled.procedures.iter().find(|procedure| procedure.entry as usize == index)
                {
                    println!("-- {} ({} registers)", procedure.name, procedure.frame_size);
                }
                println!("{index:5}  line {:<4} {instruction:?}", place.line);
            }
            ExitCode::SUCCESS
        }
        Err(messages) => {
            for mut message in messages {
                message.locate(&path, &source);
                eprint!("{message}");
            }
            ExitCode::from(1)
        }
    }
}
