use polar_core::parser;
use polar_core::parser::Line;
use polar_formatter::pretty_print::*;
use std::io::{self, Read};

fn main() {
    let mut file: String = String::new();
    io::stdin().read_to_string(&mut file).unwrap();

    let lines = parser::parse_lines(1, &file);

    let print_context = PrettyPrintContext::new(file.clone());

    if let Ok(lines) = lines {
        for line in lines {
            println!("{}", line.to_pretty_string(&print_context));
        }
    }
}
