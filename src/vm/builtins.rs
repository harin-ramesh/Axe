use super::vm::{Heap, Value};

/// Native functions receive their args and `&mut Heap` so they can allocate
/// heap objects (e.g. `range` building a list).
pub type NativeFn = fn(&[Value], &mut Heap) -> Value;

pub fn builtins() -> &'static [(&'static str, NativeFn)] {
    &[
        ("print", native_print),
        ("println", native_println),
        ("range", native_range),
        ("len", native_len),
    ]
}

fn native_print(args: &[Value], heap: &mut Heap) -> Value {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ")
        }
        print!("{}", arg.display(heap));
    }
    Value::Null
}

fn native_println(args: &[Value], heap: &mut Heap) -> Value {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ")
        }
        print!("{}", arg.display(heap));
    }
    println!();
    Value::Null
}

/// `range(end)` -> [0, 1, .., end-1]; `range(start, end)` -> [start, .., end-1].
fn native_range(args: &[Value], heap: &mut Heap) -> Value {
    let (start, end) = match args {
        [Value::Int(end)] => (0, *end),
        [Value::Int(start), Value::Int(end)] => (*start, *end),
        _ => panic!("range expects 1 or 2 integer arguments"),
    };
    let items: Vec<Value> = (start..end).map(Value::Int).collect();
    heap.alloc_list(items)
}

/// `len(x)` -> length of a list or string.
fn native_len(args: &[Value], heap: &mut Heap) -> Value {
    match args {
        [value] => Value::Int(heap.value_len(value)),
        _ => panic!("len expects exactly 1 argument"),
    }
}
