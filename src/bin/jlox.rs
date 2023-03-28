// rlox: Lox interpreter/compiler in Rust.
//    Copyright 2023 James Smith <j@mes.sh>
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.

use std::env::args;
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};

use rlox::errors::{ParseError, RloxError};
use rlox::interpreter::Interpreter;
use rlox::scanner::Scanner;

fn main() -> Result<(), Box<dyn Error>> {
    let cmdline: Vec<String> = args().collect();

    match cmdline.len() {
        // Too many arguments
        len if len > 2 => {
            eprintln!("Usage: jlox [script]");
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
fn run(input: &str) -> Result<(), RloxError> {
    // TODO: Clean this up.
    let mut scanner = Scanner::new(input);
    scanner.scan_tokens()?;

    // Parse the input and evaluate expressions
    let program = scanner.into_parser().parse();

    // Does this work?
    // We want to print all parsing errors
    let has_error = program.iter().any(|i| i.is_err());
    let errors = program.iter().filter(|err| err.is_err()).map(|err| {
        println!("{err}");
    });
    if has_error {
        println!("Exiting.");
        return Err(ParseError::ParseFailure);
    }

    let interpreter = Interpreter::new();
    interpreter.interpret(program)?;
    Ok(())
}

// Reads a file in and runs it.
fn run_file(filename: &str) -> Result<(), Box<dyn Error>> {
    let file_handle = File::open(filename)?;
    let buf = io::read_to_string(file_handle)?;

    run(&buf)?;
    Ok(())
}

// Interactive REPL prompt.
// Runs code line-by-line.
fn run_prompt() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();

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
        if let Err(err) = run(&buf) {
            println!("{err}");
        }
    }

    Ok(())
}
