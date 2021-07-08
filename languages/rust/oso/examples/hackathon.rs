use ctor::ctor;
use oso::{is_allowed, load_file};

load_file!("test.polar");

fn main() {
    let x = is_allowed!("sam", "hack", "Polar");
    assert!(x);

    let x = is_allowed!("sam", "upset", "Polar");
    assert!(!x);
}
