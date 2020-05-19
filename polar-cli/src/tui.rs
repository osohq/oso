use crossterm::execute;
use crossterm::{cursor, event, terminal};
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
// use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, Text, Widget};
use tui::Terminal;

use polar::{
    types::{QueryEvent, ToPolarString},
    Polar, Query,
};

use std::io;
use std::io::Write;

/// App holds the state of the application
#[derive(Default)]
struct App {
    query: Option<Query>,

    /// History of recorded messages
    messages: Vec<String>,

    bindings: Vec<String>,
    choices: Vec<String>,
    goals: Vec<String>,
}

pub fn main() -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    let res = std::panic::catch_unwind(move || -> anyhow::Result<()> {
        execute!(stdout, terminal::EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Create default app state
        let mut app = App::default();

        let mut polar = Polar::new();
        let mut repl = crate::repl::Repl::new();
        let mut rl = rustyline::Editor::<()>::new();
        let mut args = std::env::args();
        let _ = args.next(); // skip the binary filename
        crate::cli::load_files(&mut polar, &mut args)?;
        'tui: loop {
            // draw UI
            terminal.draw(|mut f| {
                // Split the UI into two equal chunks
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Percentage(10),
                            Constraint::Percentage(30),
                            Constraint::Percentage(60),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                // First block: REPL
                let block = Block::default().title("REPL").borders(Borders::ALL);
                f.render_widget(block, chunks[1]);

                let messages = app.messages.iter().map(|m| Text::raw(m));
                let message_list = List::new(messages)
                    .block(Block::default().borders(Borders::ALL).title("Messages"));
                f.render_widget(message_list, chunks[1]);

                // Second block: VM information
                let block = Block::default()
                    .title("VM Information")
                    .borders(Borders::ALL);
                f.render_widget(block, chunks[2]);

                let vm_info_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(1)
                    .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                    .split(chunks[2]);

                // Split VM info into 3 chunks
                let vm_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(vm_info_chunks[0]);

                let messages = app.bindings.iter().map(|m| Text::raw(m));
                let message_list = List::new(messages)
                    .block(Block::default().borders(Borders::ALL).title("Bindings"));
                f.render_widget(message_list, vm_info_chunks[1]);

                let messages = app.choices.iter().map(|m| Text::raw(m));
                let message_list = List::new(messages)
                    .block(Block::default().borders(Borders::ALL).title("Choices"));
                f.render_widget(message_list, vm_chunks[0]);

                let messages = app.goals.iter().map(|m| Text::raw(m));
                let message_list = List::new(messages)
                    .block(Block::default().borders(Borders::ALL).title("Goals"));
                f.render_widget(message_list, vm_chunks[1]);
            })?;

            execute!(terminal.backend_mut(), cursor::MoveTo(0, 0))?;
            execute!(
                terminal.backend_mut(),
                terminal::Clear(terminal::ClearType::CurrentLine)
            )?;

            if let Some(ref mut query) = app.query {
                write!(
                    terminal.backend_mut(),
                    "Press any key to continue, or \"q\" to stop query"
                )?;
                io::stdout().flush().ok();
                // wait for key press
                loop {
                    match event::read()? {
                        event::Event::Key(e) if e == event::KeyCode::Char('q').into() => {
                            app = App::default();
                            continue 'tui;
                        }
                        event::Event::Key(_) => {
                            break;
                        }
                        _ => {}
                    }
                }
                app.bindings.clear();
                app.choices.clear();
                app.goals.clear();
                let res = query.next();
                // set current state
                for binding in &query.vm().bindings {
                    app.bindings.push(format!("{}", binding));
                }
                for choice in &query.vm().choices {
                    app.choices.push(format!("{}", choice));
                }
                for goal in &query.vm().goals {
                    app.goals.push(format!("{}", goal));
                }
                match res {
                    Some(Ok(QueryEvent::Done)) => {
                        app.messages.push("False".to_string());
                        app.query = None;
                    }
                    Some(Ok(QueryEvent::Result { bindings })) => {
                        app.messages.push("True".to_string());
                        for (k, v) in bindings {
                            app.messages
                                .push(format!("  {} = {}", k.to_polar(), v.to_polar()));
                        }
                    }
                    Some(Ok(QueryEvent::BreakPoint)) => {}
                    Some(Ok(QueryEvent::ExternalCall {
                        call_id, attribute, ..
                    })) => {
                        // get input
                        execute!(terminal.backend_mut(), cursor::MoveTo(1, 1))?;
                        let input: String =
                            match rl.readline(&format!("{} = ", attribute.to_polar())) {
                                Ok(input) => input,
                                Err(_) => {
                                    app.query = None;
                                    continue;
                                }
                            };
                        let result = if input.is_empty() {
                            None
                        } else {
                            polar::parser::parse_term(&input).ok()
                        };
                        polar.external_call_result(query, call_id, result);
                    }
                    Some(Ok(e)) => app.messages.push(format!("Event: {:?}", e)),
                    Some(Err(e)) => {
                        app.messages.push(format!("Error: {:?}", e));
                        app.query = None;
                    }
                    None => {
                        app.query = None;
                    }
                }
            } else {
                write!(terminal.backend_mut(), "Enter query:")?;
                io::stdout().flush().ok();
                // Put the cursor back inside the input box
                execute!(terminal.backend_mut(), cursor::MoveTo(1, 1))?;

                // get input
                let input: String = match repl.input(">> ") {
                    Ok(input) => input,
                    Err(_) => {
                        // eprintln!("Readline error: {}", e);
                        break;
                    }
                };
                let mut query = polar.new_query(&input).unwrap();
                query.debug(true);
                app.query = Some(query);
                app.bindings.clear();
                app.choices.clear();
                app.goals.clear();
                app.messages.clear();
            }
        }
        Ok(())
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
    println!("Exiting TUI");
    Ok(())
}
