use ctor::ctor;
use procosious::{is_allowed, load_file};
use std::time::Instant;

use oso::{magic_is_allowed, PolarClass, GLOBAL_OSO};

load_file!("test.polar");

#[derive(Clone, PolarClass)]
struct Resource {
    id: i64,
    bar: String,
}

fn main() {
    let _ = tracing_subscriber::fmt::try_init();

    // base_case();
    // precompute_test();
    precompile_test();
}

fn base_case() {
    println!("BASE CASE GO");
    let oso = GLOBAL_OSO.lock().unwrap();
    #[derive(Clone, PolarClass)]
    struct Resource {
        #[polar(attribute)]
        id: i64,
        #[polar(attribute)]
        bar: String,
    }
    let resource1 = Resource {
        id: 1,
        bar: "abc".to_string(),
    };
    let resource233 = Resource {
        id: 233,
        bar: "abc".to_string(),
    };
    println!("Can sam get resource 1?");
    let now = Instant::now();
    let res1 = oso.is_allowed("sam", "get", resource1.clone()).unwrap();
    println!("Elapsed: {:?}", now.elapsed());

    println!("{:#?}", res1);
    println!("Can sam get resource 233?");
    let now = Instant::now();
    let res233 = oso.is_allowed("sam", "get", resource233).unwrap();
    println!("Elapsed: {:?}", now.elapsed());

    println!("{:#?}", res233);
    println!("Can sam get resource 1?");
    let now = Instant::now();
    let res1_again = oso.is_allowed("sam", "get", resource1).unwrap();
    println!("Elapsed: {:?}", now.elapsed());
    println!("{:#?}", res1_again);
    assert!(!res1 && res233 && !res1_again);
}

fn precompile_test() {
    println!("PRECOMPILE GO");
    let resource1 = Resource {
        id: 1,
        bar: "abc".to_string(),
    };
    let resource233 = Resource {
        id: 233,
        bar: "abc".to_string(),
    };
    println!("Can sam get resource 1?");
    let now = Instant::now();
    assert!(!is_allowed!("sam", "get", resource1));
    println!("Elapsed: {:?}", now.elapsed());
    println!("Can sam get resource 233?");
    let now = Instant::now();
    assert!(is_allowed!("sam", "get", resource233));
    println!("Elapsed: {:?}", now.elapsed());
}

fn precompute_test() {
    println!("PRECOMPUTE GO");

    #[derive(Clone, PolarClass)]
    struct Resource {
        #[polar(attribute)]
        id: i64,
        #[polar(attribute)]
        bar: String,
    }
    let resource1 = Resource {
        id: 1,
        bar: "abc".to_string(),
    };
    let resource233 = Resource {
        id: 233,
        bar: "abc".to_string(),
    };
    println!("Can sam get resource 1?");
    let now = Instant::now();
    let res1 = magic_is_allowed("sam", "get", resource1.clone()).unwrap();
    println!("Elapsed: {:?}", now.elapsed());

    println!("{:#?}", res1);
    println!("Can sam get resource 233?");
    let now = Instant::now();
    let res233 = magic_is_allowed("sam", "get", resource233).unwrap();
    println!("Elapsed: {:?}", now.elapsed());

    println!("{:#?}", res233);
    println!("Can sam get resource 1?");
    let now = Instant::now();
    let res1_again = magic_is_allowed("sam", "get", resource1).unwrap();
    println!("Elapsed: {:?}", now.elapsed());
    println!("{:#?}", res1_again);
    assert!(!res1 && res233 && !res1_again);
}
