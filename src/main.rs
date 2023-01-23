use std::env::args;
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};

mod errors;
mod scanner;
mod tokens;

use errors::RloxError;

fn main() -> Result<(), Box<dyn Error>> {
    let cmdline: Vec<String> = args().collect();

    match cmdline.len() {
        l if l > 2 => {
            eprintln!("Usage: rlox [script]");
            return Err(Box::new(RloxError::CmdlineError(String::from(
                "Too many arguments.",
            ))));
        }
        l if l == 2 => run_file(&cmdline[1])?,
        _ => run_prompt()?,
    }

    Ok(())
}

fn run(buf: &str) -> Result<(), Box<dyn Error>> {
    let scanner = scanner::Scanner::new(buf);

    let tokens = scanner.scan_tokens()?;
    for token in tokens {
        println!("{}", token);
    }
    Ok(())
}

// Reads a file in and runs it.
fn run_file(filename: &str) -> Result<(), Box<dyn Error>> {
    // scanner::Scanner::read_file(filename)?;
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
            Ok(1) => print!("\nUse ^D to close REPL."),
            Ok(_) => (),
            // Ignore any errors for now.
            Err(e) => eprint!("{:?}", e),
        }

        // Run user's input
        // Don't kill the user's session if they make a mistake.
        // Print the error.
        let res = run(&buf);
        if let Err(e) = res {
            println!("{}", e);
        }
    }

    Ok(())
}
