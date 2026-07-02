use super::vm::{Heap, Value};

pub type NativeFn = fn(&[Value], &Heap) -> Value;

pub fn builtins() -> &'static [(&'static str, NativeFn)] {
    &[("print", native_print), ("println", native_println)]
}

fn native_print(args: &[Value], heap: &Heap) -> Value {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ")
        }
        print!("{}", arg.display(heap));
    }
    Value::Null
}

fn native_println(args: &[Value], heap: &Heap) -> Value {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ")
        }
        print!("{}", arg.display(heap));
    }
    println!("");
    Value::Null
}
