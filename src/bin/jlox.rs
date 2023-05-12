// rlox: Lox interpreter/compiler in Rust.
// Copyright (C) 2023  James Smyle <j@mes.sh>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::env::args;
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};

use rlox::errors::{ParseError, RloxError};
use rlox::interpreter::Interpreter;
use rlox::scanner::Scanner;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let cmdline: Vec<String> = args().collect();

    match cmdline.len() {
        // Too many arguments
        len if len > 2 => {
            log::error!("Usage: jlox [script]");
            return Err(Box::new(RloxError::Cmdline(
                "Too many arguments.".to_string(),
            )));
        }
        // Filename given
        len if len == 2 => run_file(&cmdline[1])?,
        // No filename, run REPL
        _ => run_prompt()?,
    }

    Ok(())
}

// Scans, Parses, and evaluates input.
fn run(interpreter: &mut Interpreter, input: &str) -> Result<(), RloxError> {
    // TODO: Clean this up.
    let mut scanner = Scanner::new();
    scanner.scan_tokens(input)?;

    log::debug!("{:?}", scanner);

    // Parse the input and evaluate expressions
    let program = scanner.into_parser().parse();

    // Print all errors we've found from parsing
    for res in &program {
        if let Err(err) = res {
            log::error!("{}", err);
        }
    }
    let has_error = program.iter().any(|i| i.is_err());
    if has_error {
        return Err(RloxError::Parse(ParseError::ParseFailure(
            "Exiting.".to_string(),
        )));
    }

    // Collect just the successful parses.
    // This is either everything, or nothing. Because we exited in the last step
    let program: Result<Vec<_>, RloxError> = program.into_iter().collect();

    // debugging
    if let Ok(program) = &program {
        for stmt in program {
            log::debug!("{:?}", stmt);
        }
    }

    // Semantic Analysis

    // Interpret
    interpreter.interpret(program?)
}

// Reads a file in and runs it.
fn run_file(filename: &str) -> Result<(), Box<dyn Error>> {
    let file_handle = File::open(filename)?;
    let buf = io::read_to_string(file_handle)?;

    let mut interpreter = Interpreter::new();
    run(&mut interpreter, &buf)?;
    Ok(())
}

// Interactive REPL prompt.
// Runs code line-by-line.
fn run_prompt() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let mut interpreter = Interpreter::new();

    loop {
        let mut buf = String::new();

        // Flush prompt to screen.
        print!("> ");
        io::stdout().flush().unwrap();

        // Read in user's input
        let line = stdin.read_line(&mut buf);

        // Break out of the REPL when Control-D is pressed.
        match line {
            Ok(0) => break,
            Ok(1) => println!("\nUse ^D to close REPL."),
            Ok(_) => (),
            // Ignore any errors for now.
            Err(e) => print!("error: {e:?}"),
        }

        // Run user's input
        // Don't kill the user's session if they make a mistake.
        if let Err(err) = run(&mut interpreter, &buf) {
            log::error!("{err}");
        }
    }

    Ok(())
}
