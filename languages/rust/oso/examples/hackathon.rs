use ctor::ctor;
use procosious::{is_allowed, load_file};

use oso::{magic_is_allowed, PolarClass};

load_file!("test.polar");

#[derive(Clone, PolarClass)]
struct PolarResource {
    #[polar(attribute)]
    id: i64,
}

#[derive(Clone, PolarClass)]
struct Resource {
    id: i64,
    bar: String,
}

fn main() {
    let _ = tracing_subscriber::fmt::try_init();
    let resource1 = Resource {
        id: 1,
        bar: "abc".to_string(),
    };
    let resource233 = Resource {
        id: 233,
        bar: "abc".to_string(),
    };
    println!("Can sam get resource 1?");
    assert!(!is_allowed!("sam", "get", resource1));
    println!("Can sam get resource 233?");
    assert!(is_allowed!("sam", "get", resource233));

    // precompute_test();
}

fn precompute_test() {
    let resource1 = PolarResource { id: 1 };
    let resource233 = PolarResource { id: 233 };
    println!("Can sam get resource 1?");
    let res1 = magic_is_allowed("sam", "get", resource1.clone()).unwrap();
    println!("{:#?}", res1);
    println!("Can sam get resource 233?");
    let res233 = magic_is_allowed("sam", "get", resource233).unwrap();
    println!("{:#?}", res233);
    println!("Can sam get resource 1?");
    let res1_again = magic_is_allowed("sam", "get", resource1).unwrap();
    println!("{:#?}", res1_again);
    assert!(!res1 && res233 && !res1_again);
}
