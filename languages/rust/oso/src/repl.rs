//! Code for making interactive Oso queries from a REPL.

use clap::{Arg, Command};
use rustyline::error::ReadlineError;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Editor;
use rustyline_derive::{Completer, Helper, Highlighter, Hinter};

use oso::Oso;

use std::env;
use std::fs::OpenOptions;

/// Build the App for handling command line parameters
fn build_app() -> Command<'static> {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Oso REPL. See https://docs.osohq.com/rust/reference/tooling/repl.html")
        .arg(
            Arg::with_name("FILES")
                .multiple(true)
                .multiple_values(true)
                .help("Specify one or more .polar files to load"),
        )
}

/// Attempt to create a new temporary directory to store
/// and track the oso history
pub fn try_create_history_file() -> Option<std::path::PathBuf> {
    let mut dir = env::temp_dir();
    dir.push(".oso-history");
    match OpenOptions::new().write(true).create_new(true).open(&dir) {
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Some(dir),
        Ok(_) => Some(dir),
        Err(e) => {
            eprintln!("Error creating history file: {}", e);
            None
        }
    }
}

/// Provides input validation.
///
/// Currently, is only used to determine whether a line is
/// incomplete (missing a ';').
#[derive(Completer, Helper, Highlighter, Hinter)]
struct InputValidator {}

impl Validator for InputValidator {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult, ReadlineError> {
        let _input = ctx.input();
        // if !input.ends_with(';') {
        //     return Ok(ValidationResult::Incomplete);
        // }
        Ok(ValidationResult::Valid(None))
    }
}

pub struct Repl {
    editor: Editor<InputValidator>,
    plain_editor: Editor<()>,
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
        let history = try_create_history_file();
        if let Some(ref dir) = history {
            if let Err(error) = editor.load_history(dir) {
                eprintln!("loading history failed: {}", error);
            }
        }

        Self {
            editor,
            history,
            plain_editor: Editor::new(),
        }
    }

    pub fn oso_input(&mut self, prompt: &str) -> anyhow::Result<String> {
        let input = self.editor.readline(prompt)?;
        self.editor.add_history_entry(input.as_str());
        Ok(input)
    }

    pub fn plain_input(&mut self, prompt: &str) -> anyhow::Result<String> {
        Ok(self.plain_editor.readline(prompt)?)
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
    tracing_subscriber::fmt::init();
    let mut repl = Repl::new();
    let mut oso = Oso::new();

    let matches = build_app().get_matches();
    if matches.is_present("FILES") {
        oso.load_files(matches.values_of("FILES").unwrap().collect())?;
    }

    loop {
        // get input
        let input: String = match repl.oso_input("query> ") {
            Ok(input) => input,
            Err(e) => {
                eprintln!("Readline error: {}", e);
                break;
            }
        };

        let query = match oso.query(&input) {
            Err(e) => {
                println!("{}", e);
                continue;
            }
            Ok(q) => q,
        };
        let mut has_result = false;
        for res in query {
            has_result = true;
            if let Ok(res) = res {
                if res.is_empty() {
                    println!("true");
                } else {
                    for (var, value) in res.iter_bindings() {
                        println!("{} = {}", var, value);
                    }
                }
            } else {
                println!("{}", res.expect_err("error"))
            }
        }
        if !has_result {
            println!("false")
        }
    }
    Ok(())
}
