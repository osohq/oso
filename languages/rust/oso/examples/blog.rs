//! Code for showing blog progress

use oso::{Class, Oso, ToPolar};
use tracing::{info, instrument};

#[instrument]
fn example_zero() -> anyhow::Result<()> {
    let mut oso = Oso::new();

    oso.load_str("x_is_one(x) if x = 1;")?;
    let mut query = oso.query_rule("x_is_one", (1,))?;
    let _ = query.next().expect("no results").expect("resulted in err");

    info!("Example complete");
    Ok(())
}

#[instrument]
fn example_one() -> anyhow::Result<()> {
    let mut oso = Oso::new();

    #[derive(PolarClass)]
    struct Foo {}

    oso.load_str("is_a_foo(_x: Foo);")?;
    oso.register_class(Foo::get_polar_class())?;

    let example_foo = Foo {};
    let mut query = oso.query_rule("is_a_foo", (example_foo,))?;
    let _ = query.next().expect("no results").expect("resulted in err");

    info!("Example complete");
    Ok(())
}

#[instrument]
fn example_two() -> anyhow::Result<()> {
    let mut oso = Oso::new();

    struct Foo {
        x: u32,
    }
    impl ToPolar for Foo {}

    oso.load_str("foo_x_is_one(foo: Foo) if foo.x = 1;")?;
    oso.register_class(
        Class::builder::<Foo>()
            .name("Foo")
            .add_attribute_getter("x", |f| f.x)
            .build(),
    )?;

    let example_foo = Foo { x: 1 };
    let mut query = oso.query_rule("foo_x_is_one", (example_foo,))?;
    let _ = query.next().expect("no results").expect("resulted in err");

    info!("Example complete");
    Ok(())
}

#[instrument]
fn example_three() -> anyhow::Result<()> {
    let mut oso = Oso::new();

    struct Foo {
        x: u32,
    }

    impl Foo {
        fn x_plus_y(&self, y: u32) -> u32 {
            self.x + y
        }
    }

    impl ToPolar for Foo {}

    oso.load_str("x_plus_y_is_two(foo: Foo) if foo.x_plus_y(1) = 2;")?;
    oso.register_class(
        Class::builder::<Foo>()
            .name("Foo")
            .add_attribute_getter("x", |f| f.x)
            .add_method("x_plus_y", Foo::x_plus_y)
            .build(),
    )?;

    let example_foo = Foo { x: 1 };
    let mut query = oso.query_rule("x_plus_y_is_two", (example_foo,))?;
    let _ = query.next().expect("no results").expect("resulted in err");

    info!("Example complete");
    Ok(())
}

#[instrument]
fn example_four() -> anyhow::Result<()> {
    let mut oso = Oso::new();

    struct Foo {
        x: u32,
    }

    impl Foo {
        fn x_plus_y(&self, y: u32) -> u32 {
            self.x + y
        }

        fn get_z() -> u32 {
            3
        }
    }

    impl ToPolar for Foo {}

    oso.load_str("x_plus_y_plus_z_is_five(foo: Foo) if foo.x_plus_y(1) + Foo.get_z() = 5;")?;
    oso.register_class(
        Class::builder::<Foo>()
            .name("Foo")
            .add_attribute_getter("x", |f| f.x)
            .add_method("x_plus_y", Foo::x_plus_y)
            .add_class_method("get_z", Foo::get_z)
            .build(),
    )?;

    let example_foo = Foo { x: 1 };
    let mut query = oso.query_rule("x_plus_y_plus_z_is_five", (example_foo,))?;
    let _ = query.next().expect("no results").expect("resulted in err");

    info!("Example complete");
    Ok(())
}

pub fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    example_zero()?;
    example_one()?;
    example_two()?;
    example_three()?;
    example_four()?;

    Ok(())
}

#[test]
fn test() {
    main().unwrap();
}
