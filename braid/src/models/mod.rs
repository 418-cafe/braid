use sqlx::Row;

mod commit;

trait Sealed {}


#[allow(private_bounds)]
pub trait Foo : Sealed {}

trait FooEx : Foo {
    fn new() -> Self;
}

impl<F: Foo> FooEx for F {
    fn new() -> Self {
        unimplemented!()
    }
}

fn foo<F: Foo>(f: F) {
    <F as FooEx>::new();
}