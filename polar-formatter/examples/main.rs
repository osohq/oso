extern crate polar_formatter;

use polar_formatter::format;
use std::io::{self, Read};

fn main() {
    let mut file: String = String::new();
    io::stdin().read_to_string(&mut file).unwrap();

    let result = format(file);
    if let Some(res) = result {
        println!("{}", res);
    } else {
        println!("PARSE ERROR");
    }
}
