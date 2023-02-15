// rlox: Lox interpreter/compiler in Rust.
// Copyright (C) 2023 James Smith

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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

// Lexes the scanner and evaluates input.
fn run(scanner: &mut Scanner) -> Result<(), Box<dyn Error>> {
    scanner.scan_tokens()?;
    let mut p = Parser::new(scanner.get_tokens());

    if let Some(expr) = p.parse() {
        println!("{expr:?}");
    }

    Ok(())
}

// Reads a file in and runs it.
fn run_file(filename: &str) -> Result<(), Box<dyn Error>> {
    let file_handle = File::open(filename)?;
    let buf = io::read_to_string(file_handle)?;

    let mut scanner = Scanner::new(&buf);
    run(&mut scanner)?;
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

        // Create a scanner from user's input
        let mut scanner = Scanner::new(&buf);

        // Run user's input
        // Don't kill the user's session if they make a mistake.
        // Print the error.
        let res = run(&mut scanner);
        if let Err(e) = res {
            println!("{e}");
        }
    }

    Ok(())
}