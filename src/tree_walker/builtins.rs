//! Built-in native functions for the interpreter.
//!
//! Native functions are implemented here and called through the interpreter.
//! All native functions receive a &Context for string interning capabilities.

use crate::ast::Literal;
use crate::context::Context;

use super::environment::{EnvRef, Environment};
use super::interpreter::EvalSignal;
use super::value::Value;

/// Initialize all built-in functions and classes in the global environment.
pub fn init_builtins(ctx: &Context, globals: &EnvRef) {
    let mut env = globals.borrow_mut();

    // Global functions
    let print_sym = ctx.intern("print");
    let type_sym = ctx.intern("type");
    let range_sym = ctx.intern("range");

    env.set(print_sym, Value::NativeFunction(print_sym, native_print));
    env.set(type_sym, Value::NativeFunction(type_sym, native_type));
    env.set(range_sym, Value::NativeFunction(range_sym, native_range));

    // We need to drop the borrow before creating class environments
    drop(env);

    // String class
    let string_class = Environment::new();
    {
        let mut class_env = string_class.borrow_mut();
        let len_sym = ctx.intern("len");
        let concat_sym = ctx.intern("concat");
        class_env.set(len_sym, Value::NativeFunction(len_sym, string_len));
        class_env.set(concat_sym, Value::NativeFunction(concat_sym, string_concat));
    }
    let string_sym = ctx.intern("String");
    globals
        .borrow_mut()
        .set(string_sym, Value::Object(string_class));

    // List class
    let list_class = Environment::new();
    {
        let mut class_env = list_class.borrow_mut();
        let len_sym = ctx.intern("len");
        let concat_sym = ctx.intern("concat");
        let push_sym = ctx.intern("push");
        let get_sym = ctx.intern("get");
        class_env.set(len_sym, Value::NativeFunction(len_sym, list_len));
        class_env.set(concat_sym, Value::NativeFunction(concat_sym, list_concat));
        class_env.set(push_sym, Value::NativeFunction(push_sym, list_push));
        class_env.set(get_sym, Value::NativeFunction(get_sym, list_get));
    }
    let list_sym = ctx.intern("List");
    globals
        .borrow_mut()
        .set(list_sym, Value::Object(list_class));
}

/// Native print function - prints values to stdout.
pub fn native_print(ctx: &Context, args: &[Value]) -> Result<Value, EvalSignal> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", arg.display(ctx));
    }
    println!();
    Ok(Value::Literal(Literal::Null))
}

/// Native type function - returns the type name as a string.
pub fn native_type(ctx: &Context, args: &[Value]) -> Result<Value, EvalSignal> {
    if args.len() != 1 {
        return Err("type expects exactly 1 argument".into());
    }
    let type_name = match &args[0] {
        Value::Literal(lit) => match lit {
            Literal::Null => "null",
            Literal::Bool(_) => "bool",
            Literal::Int(_) => "int",
            Literal::Float(_) => "float",
            Literal::Str(_) => "string",
        },
        Value::List(_) => "list",
        Value::Function(..) => "function",
        Value::NativeFunction(..) => "function",
        Value::Object(..) => "object",
    };
    Ok(Value::Literal(Literal::Str(ctx.intern(type_name))))
}

/// Native range function - creates a list of integers.
pub fn native_range(_ctx: &Context, args: &[Value]) -> Result<Value, EvalSignal> {
    match args.len() {
        1 => match &args[0] {
            Value::Literal(Literal::Int(n_val)) => {
                if *n_val < 0 {
                    return Err("range expects non-negative integer".into());
                }
                let values: Vec<Value> = (0..*n_val)
                    .map(|i| Value::Literal(Literal::Int(i)))
                    .collect();
                Ok(Value::List(values))
            }
            _ => Err("range expects integer argument".into()),
        },
        2 => match (&args[0], &args[1]) {
            (Value::Literal(Literal::Int(start)), Value::Literal(Literal::Int(end))) => {
                let values: Vec<Value> = (*start..*end)
                    .map(|i| Value::Literal(Literal::Int(i)))
                    .collect();
                Ok(Value::List(values))
            }
            _ => Err("range expects integer arguments".into()),
        },
        3 => match (&args[0], &args[1], &args[2]) {
            (
                Value::Literal(Literal::Int(start)),
                Value::Literal(Literal::Int(end)),
                Value::Literal(Literal::Int(step)),
            ) => {
                if *step == 0 {
                    return Err("range step cannot be zero".into());
                }
                let mut values = Vec::new();
                let mut current = *start;
                if *step > 0 {
                    while current < *end {
                        values.push(Value::Literal(Literal::Int(current)));
                        current += step;
                    }
                } else {
                    while current > *end {
                        values.push(Value::Literal(Literal::Int(current)));
                        current += step;
                    }
                }
                Ok(Value::List(values))
            }
            _ => Err("range expects integer arguments".into()),
        },
        _ => Err("range expects 1, 2, or 3 arguments".into()),
    }
}

// =============================================================================
// String methods
// =============================================================================

/// String.len() - returns the length of the string.
pub fn string_len(_ctx: &Context, args: &[Value]) -> Result<Value, EvalSignal> {
    // args[0] is self (the string)
    if args.len() != 1 {
        return Err("len() takes no arguments".into());
    }
    match &args[0] {
        Value::Literal(Literal::Str(s)) => {
            let str_val = _ctx.resolve(*s);
            Ok(Value::Literal(Literal::Int(str_val.len() as i64)))
        }
        _ => Err("len() called on non-string".into()),
    }
}

/// String.concat(...) - concatenates strings.
pub fn string_concat(ctx: &Context, args: &[Value]) -> Result<Value, EvalSignal> {
    // args[0] is self (the string), rest are arguments to concat
    if args.is_empty() {
        return Err("concat() called without self".into());
    }
    let Value::Literal(Literal::Str(s)) = &args[0] else {
        return Err("concat() called on non-string".into());
    };
    let mut result = ctx.resolve(*s);
    for arg in &args[1..] {
        match arg {
            Value::Literal(Literal::Str(other)) => {
                result.push_str(&ctx.resolve(*other));
            }
            _ => return Err("concat() requires string arguments".into()),
        }
    }
    Ok(Value::Literal(Literal::Str(ctx.intern(&result))))
}

// =============================================================================
// List methods
// =============================================================================

/// List.len() - returns the length of the list.
pub fn list_len(_ctx: &Context, args: &[Value]) -> Result<Value, EvalSignal> {
    // args[0] is self (the list)
    if args.len() != 1 {
        return Err("len() takes no arguments".into());
    }
    match &args[0] {
        Value::List(items) => Ok(Value::Literal(Literal::Int(items.len() as i64))),
        _ => Err("len() called on non-list".into()),
    }
}

/// List.push(item) - returns a new list with the item appended.
pub fn list_push(_ctx: &Context, args: &[Value]) -> Result<Value, EvalSignal> {
    // args[0] is self (the list), args[1] is the item to push
    if args.len() != 2 {
        return Err("push() requires exactly 1 argument".into());
    }
    match &args[0] {
        Value::List(items) => {
            let mut new_list = items.clone();
            new_list.push(args[1].clone());
            Ok(Value::List(new_list))
        }
        _ => Err("push() called on non-list".into()),
    }
}

/// List.get(index) - returns the item at the given index.
pub fn list_get(_ctx: &Context, args: &[Value]) -> Result<Value, EvalSignal> {
    // args[0] is self (the list), args[1] is the index
    if args.len() != 2 {
        return Err("get() requires exactly 1 argument".into());
    }
    match (&args[0], &args[1]) {
        (Value::List(items), Value::Literal(Literal::Int(i))) => {
            // Handle negative indices (Python-style)
            let index = if *i < 0 { items.len() as i64 + i } else { *i };
            if index < 0 || index >= items.len() as i64 {
                return Err("list index out of bounds".into());
            }
            Ok(items[index as usize].clone())
        }
        (Value::List(_), _) => Err("get() requires integer index".into()),
        _ => Err("get() called on non-list".into()),
    }
}

/// List.concat(...) - concatenates lists.
pub fn list_concat(_ctx: &Context, args: &[Value]) -> Result<Value, EvalSignal> {
    // args[0] is self (the list), rest are lists to concat
    if args.is_empty() {
        return Err("concat() called without self".into());
    }
    let Value::List(items) = &args[0] else {
        return Err("concat() called on non-list".into());
    };
    let mut result = items.clone();
    for arg in &args[1..] {
        match arg {
            Value::List(other) => result.extend(other.clone()),
            _ => return Err("concat() requires list arguments".into()),
        }
    }
    Ok(Value::List(result))
}
