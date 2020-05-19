//! Terminal UI for Polar queries

use crossterm::execute;
use crossterm::{cursor, event, terminal};
use tui::backend::{self, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::terminal::Frame;
use tui::widgets::{Block, Borders, List, Text};
use tui::Terminal;

use super::repl;
use crate::{
    types::{QueryEvent, Symbol, Term},
    Polar, Query, ToPolarString,
};

use std::io;
use std::io::Write;

/// App holds the state of the application
pub struct App {
    polar: Polar,
    query: Option<Query>,

    /// History of recorded messages
    messages: Vec<String>,

    /// Current information to display in the TUI
    bindings: Vec<String>,
    choices: Vec<String>,
    goals: Vec<String>,

    /// allows for reading external call input
    rl: rustyline::Editor<()>,

    /// Used for reading polar query input like the REPL does
    repl: repl::Repl,
}

impl Default for App {
    fn default() -> Self {
        Self {
            query: None,
            polar: Polar::new(),
            messages: Vec::new(),
            bindings: Vec::new(),
            choices: Vec::new(),
            goals: Vec::new(),
            rl: rustyline::Editor::new(),
            repl: repl::Repl::new(),
        }
    }
}

impl App {
    pub fn new(polar: Polar) -> Self {
        Self {
            polar,
            ..Self::default()
        }
    }
    /// read in various information from the VM to update the
    /// application state
    fn update_state(&mut self) {
        if let Some(ref query) = self.query {
            let info = query.debug_info();
            for binding in &info.bindings {
                self.bindings.push(format!("{}", binding));
            }
            for choice in &info.choices {
                self.choices.push(format!("{}", choice));
            }
            for goal in &info.goals {
                self.goals.push(format!("{}", goal));
            }
        }
    }

    /// Clear out the Vecs tracking the VM state
    fn clear_vm_state(&mut self) {
        self.bindings.clear();
        self.choices.clear();
        self.goals.clear();
    }

    /// clear all app state
    fn clear(&mut self) {
        self.clear_vm_state();
        self.messages.clear();
        self.query = None;
    }

    /// Handle an external call event
    ///
    /// Prompts the user to input a polar term as a response
    fn external_call(&mut self, attribute: Symbol) -> Option<Term> {
        // get input
        let input: String = match self.rl.readline(&format!("{} = ", attribute.to_polar())) {
            Ok(input) => input,
            Err(_) => {
                self.query = None;
                return None;
            }
        };
        if input.is_empty() {
            None
        } else {
            crate::parser::parse_term(&input).ok()
        }
    }

    /// Handles a query event
    fn result<E: std::error::Error>(
        &mut self,
        res: Option<Result<QueryEvent, E>>,
    ) -> anyhow::Result<()> {
        match res {
            Some(Ok(QueryEvent::Done)) => {
                self.messages.push("False".to_string());
                self.query = None;
            }
            Some(Ok(QueryEvent::Result { bindings })) => {
                self.messages.push("True".to_string());
                for (k, v) in bindings {
                    self.messages
                        .push(format!("  {} = {}", k.to_polar(), v.to_polar()));
                }
            }
            Some(Ok(QueryEvent::BreakPoint)) => {}
            Some(Ok(QueryEvent::ExternalCall {
                call_id, attribute, ..
            })) => {
                let result = self.external_call(attribute);
                self.polar.external_call_result(
                    self.query.as_mut().expect("app has query"),
                    call_id,
                    result,
                );
            }
            Some(Ok(e)) => self.messages.push(format!("Event: {:?}", e)),
            Some(Err(e)) => {
                self.messages.push(format!("Error: {:?}", e));
                self.query = None;
            }
            None => {
                self.query = None;
            }
        }
        Ok(())
    }

    /// Prompts the user for the next input
    ///
    /// If the application has a query, asks for a continuation (any key) or an exit ("q")
    /// until the query is finished.
    /// Maybe prompts the user to handle an external call (see `result`).
    ///
    /// If the application has no query, asks for a new query.
    fn input(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> anyhow::Result<()> {
        if self.query.is_some() {
            write!(
                terminal.backend_mut(),
                "Press any key to continue, or \"q\" to stop query"
            )?;
            io::stdout().flush().ok();
            // wait for key press
            loop {
                match event::read()? {
                    event::Event::Key(e) if e == event::KeyCode::Char('q').into() => {
                        *self = App::default();
                        return Ok(());
                    }
                    event::Event::Key(_) => {
                        break;
                    }
                    _ => {}
                }
            }
            self.clear_vm_state();
            let res = self.query.as_mut().unwrap().next();
            // set current state
            self.update_state();
            self.result(res)?;
        } else {
            write!(terminal.backend_mut(), "Enter query:")?;
            io::stdout().flush().ok();

            // get input
            let input: String = match self.repl.input(">> ") {
                Ok(input) => input,
                Err(_) => {
                    return Err(anyhow::Error::msg("Exiting"));
                }
            };
            let mut query = self.polar.new_query(&input).unwrap();
            query.debug(true);
            self.clear();
            self.query = Some(query);
        }
        Ok(())
    }
}

fn setup_ui_chunks<B: backend::Backend>(f: &Frame<B>) -> (Rect, Rect, Rect) {
    // Split the UI into three chunks
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Percentage(60),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(f.size());
    (chunks[0], chunks[1], chunks[2])
}

fn display_messages<B: backend::Backend>(
    f: &mut Frame<B>,
    title: &'static str,
    chunk: Rect,
    messages: &[String],
) {
    let message_list = List::new(messages.iter().map(Text::raw))
        .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(message_list, chunk);
}

pub fn run(mut app: App) -> anyhow::Result<()> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // catch panics so we can clean up terminal on exit
    let res = std::panic::catch_unwind(move || -> anyhow::Result<()> {
        // Raw mode allow us to capture single key presses instead
        // of waiting for a newline (e.g. we can do "press 'q' to exit").
        crossterm::terminal::enable_raw_mode()?;
        // Enter the alternate screen, this is way cleaner
        execute!(terminal.backend_mut(), terminal::EnterAlternateScreen)?;

        let mut repl_position = (0, 0);
        let mut repl_size = (0, 0);
        // This is our `run` loop
        loop {
            // draw UI
            terminal.draw(|mut f| {
                // Split the UI into three chunks
                let (messages_chunk, vm_chunk, repl_chunk) = setup_ui_chunks(&f);

                // Display messages in a block
                display_messages(&mut f, "Messages", messages_chunk, &app.messages);

                //  Display VM information in a block, subdivided further
                let vm_block = Block::default()
                    .title("VM Information")
                    .borders(Borders::ALL);
                f.render_widget(vm_block, vm_chunk);

                // Split VM chunk into 2
                //     LHS: stacks (choices + goals)
                //     RHS: bindings
                let vm_info_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(1)
                    .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                    .split(vm_chunk);

                //  VM stacks split into choices + goals
                let vm_stacks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(vm_info_chunks[0]);

                // configure chunks to display various messages
                display_messages(&mut f, "Bindings", vm_info_chunks[1], &app.bindings);
                display_messages(&mut f, "Choices", vm_stacks[0], &app.choices);
                display_messages(&mut f, "Goals", vm_stacks[1], &app.goals);

                // set to top-left of repl chunk
                repl_position = (repl_chunk.left(), repl_chunk.top());
                repl_size = (
                    repl_chunk.right() - repl_chunk.left(),
                    repl_chunk.bottom() - repl_chunk.top(),
                );
            })?;

            // do the application stuff:
            // reset the cursor and query for input

            // move cursor to the REPL position
            execute!(
                terminal.backend_mut(),
                cursor::MoveTo(repl_position.0, repl_position.1)
            )?;
            execute!(
                terminal.backend_mut(),
                terminal::Clear(terminal::ClearType::CurrentLine)
            )?;
            app.input(&mut terminal)?;
            execute!(
                terminal.backend_mut(),
                cursor::MoveTo(repl_position.0, repl_position.1)
            )?;
        }
    });

    crossterm::terminal::disable_raw_mode()?;
    execute!(io::stdout(), terminal::LeaveAlternateScreen)?;
    match res {
        Err(e) => {
            let e = e.downcast_ref::<anyhow::Error>();
            eprintln!("{}", e.expect("TUI panicked"));
        }
        Ok(Err(e)) => {
            eprintln!("{}", e);
        }
        Ok(Ok(_)) => {}
    }
    Ok(())
}

pub fn main() -> anyhow::Result<()> {
    // Create default app state
    let mut app = App::default();
    let mut args = std::env::args();
    let _ = args.next(); // skip the binary filename
    crate::cli::load_files(&mut app.polar, &mut args)?;
    run(app)
}
