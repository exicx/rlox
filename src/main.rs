use std::env::args;
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};

mod errors;
mod scanner;
mod tokens;

use errors::RloxError;
use scanner::Scanner;

fn main() -> Result<(), Box<dyn Error>> {
    let cmdline: Vec<String> = args().collect();

    match cmdline.len() {
        // Too many arguments
        l if l > 2 => {
            eprintln!("Usage: rlox [script]");
            return Err(Box::new(RloxError::CmdlineError(String::from(
                "Too many arguments.",
            ))));
        }
        // Filename given
        l if l == 2 => run_file(&cmdline[1])?,
        // No filename, run REPL
        _ => run_prompt()?,
    }

    Ok(())
}

// Lexes the scanner and evaluates input.
fn run(scanner: &mut Scanner) -> Result<(), Box<dyn Error>> {
    scanner.scan_tokens()?;
    Ok(())
}

// Reads a file in and runs it.
fn run_file(filename: &str) -> Result<(), Box<dyn Error>> {
    // scanner::Scanner::read_file(filename)?;
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
            Ok(1) => print!("\nUse ^D to close REPL."),
            Ok(_) => (),
            // Ignore any errors for now.
            Err(e) => eprint!("{:?}", e),
        }

        // Create a scanner from user's input
        let mut scanner = Scanner::new(&buf);

        // Run user's input
        // Don't kill the user's session if they make a mistake.
        // Print the error.
        let res = run(&mut scanner);
        if let Err(e) = res {
            println!("{}", e);
        }
    }

    Ok(())
}
