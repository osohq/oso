//! Code for making interactive Polar queries from a REPL

use rustyline::error::ReadlineError;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Editor;
use rustyline_derive::{Completer, Helper, Highlighter, Hinter};

use std::env;

use crate::types::QueryEvent;
use crate::Polar;

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

pub struct Repl {
    editor: Editor<InputValidator>,
    history: Option<std::path::PathBuf>,
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}

impl Repl {
    pub fn new() -> Self {
        let h = InputValidator {};
        let mut editor = Editor::new();
        editor.set_helper(Some(h));

        // lookup or create history file
        let history = super::try_create_history_file();
        if let Some(ref dir) = history {
            if let Err(error) = editor.load_history(dir) {
                eprintln!("loading history failed: {}", error);
            }
        }

        Self { editor, history }
    }

    pub fn input(&mut self, prompt: &str) -> anyhow::Result<String> {
        let mut input = self.editor.readline(prompt)?;
        self.editor.add_history_entry(input.as_str());
        input.pop(); // remove the trailing ';'
        Ok(input)
    }
}

impl Drop for Repl {
    fn drop(&mut self) {
        if let Some(ref dir) = self.history {
            let _ = self.editor.save_history(dir);
        }
    }
}

pub fn main() -> anyhow::Result<()> {
    let mut repl = Repl::new();
    let mut polar = Polar::new();

    let mut args = env::args();
    let _ = args.next(); // skip the binary filename
    super::load_files(&mut polar, &mut args)?;
    loop {
        // get input
        let input: String = match repl.input(">> ") {
            Ok(input) => input,
            Err(e) => {
                eprintln!("Readline error: {}", e);
                break;
            }
        };
        let query = match polar.new_query(&input) {
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
            Ok(q) => q,
        };
        let mut has_result = false;
        for event in query {
            match event {
                Ok(QueryEvent::Done) => {
                    if !has_result {
                        println!("False");
                    }
                }
                Ok(QueryEvent::Result { bindings }) => {
                    println!("True");
                    for (k, v) in bindings {
                        println!("\t{:?} = {:?}", k, v);
                    }
                    has_result = true;
                }
                Ok(e) => println!("Event: {:?}", e),
                Err(e) => println!("Error: {:?}", e),
            }
        }
    }
    Ok(())
}
