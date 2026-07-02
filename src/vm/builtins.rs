use super::vm::Value;

pub type NativeFn = fn(&[Value]) -> Value;

pub fn builtins() -> &'static [(&'static str, NativeFn)] {
    &[("print", native_print), ("println", native_println)]
}

fn native_print(args: &[Value]) -> Value {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ")
        }
        print!("{}", arg.display());
    }
    Value::Null
}

fn native_println(args: &[Value]) -> Value {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ")
        }
        print!("{}", arg.display());
    }
    println!("");
    Value::Null
}
