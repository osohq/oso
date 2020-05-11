use rustyline::error::ReadlineError;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Editor;
use rustyline_derive::{Completer, Helper, Highlighter, Hinter};

use std::env;
use std::fs::{File, OpenOptions};
use std::io::Read;

use polar::types::QueryEvent;
use polar::Polar;

/// Provides input validation.
///
/// Currently, is only used to determine whether a line is
/// incomplete (missing a ';').
#[derive(Completer, Helper, Highlighter, Hinter)]
struct InputValidator {}

impl Validator for InputValidator {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult, ReadlineError> {
        let input = ctx.input();
        if !input.ends_with(';') {
            return Ok(ValidationResult::Incomplete);
        }
        Ok(ValidationResult::Valid(None))
    }
}

/// Attempt to create a new temporary directory to store
/// and track the polar history
fn try_create_history_file() -> Option<std::path::PathBuf> {
    let mut dir = env::temp_dir();
    dir.push(".polar-history");
    match OpenOptions::new().write(true).create_new(true).open(&dir) {
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            tracing::trace!("History file exists at: {:?}", dir);
            Some(dir)
        }
        Ok(_) => {
            tracing::trace!("History file created at: {:?}", dir);
            Some(dir)
        }
        Err(e) => {
            tracing::error!("Error creating history file: {}", e);
            None
        }
    }
}

fn main() -> rustyline::Result<()> {
    tracing_subscriber::fmt::init();

    let h = InputValidator {};
    let mut rl = Editor::new();
    rl.set_helper(Some(h));

    // lookup or create history file
    let maybe_history = try_create_history_file();
    if let Some(ref dir) = maybe_history {
        rl.load_history(dir)?;
    }

    // create a new polar instance
    let mut polar = Polar::new();
    let mut args = env::args();
    let _ = args.next(); // skip the filename
    for argument in args {
        println!("Loading: {}", argument);
        let mut f = File::open(argument).expect("open file");
        let mut policy = String::new();
        f.read_to_string(&mut policy).expect("read in policy");
        polar.load_str(&policy).unwrap();
    }
    polar.load_str("foo(1);foo(2);").unwrap();
    loop {
        // get input
        let mut input = match rl.readline(">> ") {
            Ok(input) if input == "exit;" => break,
            Err(e) => {
                eprintln!("Readline error: {}", e);
                break;
            }
            Ok(input) => input,
        };
        rl.add_history_entry(input.as_str());
        input.pop(); // remove the trailing ';'

        let mut query = polar.new_query(&input).unwrap();
        loop {
            match query.next() {
                Some(Ok(QueryEvent::Done)) => println!("False"),
                Some(Ok(QueryEvent::Result { bindings })) => {
                    println!("True");
                    for (k, v) in bindings {
                        println!("\t{:?} = {:?}", k, v);
                    }
                }
                Some(Ok(e)) => println!("Event: {:?}", e),
                Some(Err(e)) => println!("Error: {:?}", e),
                None => break,
            }
        }
    }
    if let Some(ref dir) = maybe_history {
        rl.save_history(dir)?;
    }
    // tracing::info!("Final state:\n{:?}", polar);
    Ok(())
}
