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

use rlox::errors::RloxError;
use rlox::parser::Parser;
use rlox::scanner::Scanner;

fn main() -> Result<(), Box<dyn Error>> {
    let cmdline: Vec<String> = args().collect();

    match cmdline.len() {
        // Too many arguments
        len if len > 2 => {
            eprintln!("Usage: jlox [script]");
            return Err(Box::new(RloxError::Cmdline(String::from(
                "Too many arguments.",
            ))));
        }
        // Filename given
        len if len == 2 => run_file(&cmdline[1])?,
        // No filename, run REPL
        _ => run_prompt()?,
    }

    Ok(())
}

// Scans, Parses, and evaluates input.
fn run(input: &str) -> Result<(), Box<dyn Error>> {
    // TODO: Clean this up.
    let mut scanner = Scanner::new(&input);
    scanner.scan_tokens()?;

    println!("{scanner:?}");

    let mut p = scanner.into_parser();

    while let Some(expr) = p.parse() {
        println!("{expr:?}");
    }

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
            Err(e) => eprint!("{e:?}"),
        }

        // Run user's input
        // Don't kill the user's session if they make a mistake.
        // Print the error.
        let res = run(&buf);
        if let Err(e) = res {
            println!("{e}");
        }
    }

    Ok(())
}
