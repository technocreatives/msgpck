use msgpck::{MsgPck, UnMsgPck};

struct Foo {
    a: bool,
    b: String,
}

#[derive(MsgPck, UnMsgPck)]
struct Bar {
    a: f32,
    foo: Foo,
}
