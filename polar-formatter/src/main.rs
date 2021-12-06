use polar_formatter::parser;
use polar_formatter::pretty_print::PrettyContext;
use polar_formatter::pretty_print::ToDoc;
use std::io::{self, Read};

fn main() {
    let mut file: String = String::new();
    io::stdin().read_to_string(&mut file).unwrap();

    // let lines = parser::parse_lines(1, &file);

    let node = parser::parse_file(1, &file);

    let mut context = PrettyContext::new(file.clone());

    let mut result = Vec::new();
    if let Ok(node) = node {
        node.to_doc(&mut context).render(80, &mut result).unwrap();
        let output = String::from_utf8(result).unwrap();
        println!("{}", output);
        println!("{}", context.print_trailing_comments());
    } else {
        print!("{}", file);
    }
}
