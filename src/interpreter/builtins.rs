//! Built-in native functions for the interpreter.

use crate::ast::Literal;

use super::value::Value;

pub fn native_print(args: &[Value]) -> Result<Value, &'static str> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }

        match arg {
            Value::Literal(s) => {
                if let Literal::Str(s) = s {
                    print!("{}", s);
                } else {
                    print!("{}", arg);
                }
            }
            _ => print!("{}", arg),
        }
    }
    println!();
    Ok(Value::Literal(Literal::Null))
}

pub fn native_len(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 1 {
        return Err("len expects exactly 1 argument");
    }
    match &args[0] {
        Value::List(items) => Ok(Value::Literal(Literal::Int(items.len() as i64))),
        Value::Literal(s) => match s {
            Literal::Str(s) => Ok(Value::Literal(Literal::Int(s.len() as i64))),
            _ => Err("len expects a list or string"),
        },
        _ => Err("len expects a list or string"),
    }
}

pub fn native_push(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 2 {
        return Err("push expects exactly 2 arguments");
    }
    match &args[0] {
        Value::List(items) => {
            let mut new_list = items.clone();
            new_list.push(args[1].clone());
            Ok(Value::List(new_list))
        }
        _ => Err("push expects a list as first argument"),
    }
}

pub fn native_get(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 2 {
        return Err("get expects exactly 2 arguments");
    }
    match (&args[0], &args[1]) {
        (Value::List(items), Value::Literal(i)) => match i {
            Literal::Int(idx) => {
                let index = if *idx < 0 {
                    (items.len() as i64 + idx) as usize
                } else {
                    *idx as usize
                };
                items.get(index).cloned().ok_or("index out of bounds")
            }
            _ => Err("get expects an integer index as second argument"),
        },
        _ => Err("get expects a list and an integer index"),
    }
}

pub fn native_type(args: &[Value]) -> Result<Value, &'static str> {
    if args.len() != 1 {
        return Err("type expects exactly 1 argument");
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
        Value::NativeFunction(..) => "native-function",
        Value::Object(..) => "object",
    };
    Ok(Value::Literal(Literal::Str(type_name.to_string())))
}

pub fn native_concat(args: &[Value]) -> Result<Value, &'static str> {
    if args.is_empty() {
        return Err("concat expects at least 1 argument");
    }

    // Check if first argument is a string or list
    match &args[0] {
        Value::Literal(s) => match s {
            Literal::Str(_) => {
                let mut result = String::new();
                for arg in args {
                    match arg {
                        Value::Literal(s) => match s {
                            Literal::Str(s) => result.push_str(s),
                            _ => {
                                return Err(
                                    "concat on strings requires all arguments to be strings",
                                )
                            }
                        },
                        _ => return Err("concat on strings requires all arguments to be strings"),
                    }
                }
                Ok(Value::Literal(Literal::Str(result)))
            }
            _ => Err("concat expects strings or lists"),
        },
        Value::List(_) => {
            // Concatenate lists
            let mut result = Vec::new();
            for arg in args {
                match arg {
                    Value::List(items) => result.extend(items.clone()),
                    _ => return Err("concat on lists requires all arguments to be lists"),
                }
            }
            Ok(Value::List(result))
        }
        _ => Err("concat expects strings or lists"),
    }
}

pub fn native_range(args: &[Value]) -> Result<Value, &'static str> {
    match args.len() {
        1 => match &args[0] {
            Value::Literal(n) => match n {
                Literal::Int(n_val) => {
                    if *n_val < 0 {
                        return Err("range expects non-negative integer");
                    }
                    let values: Vec<Value> = (0..*n_val)
                        .map(|i| Value::Literal(Literal::Int(i)))
                        .collect();

                    Ok(Value::List(values))
                }
                _ => Err("range expects integer argument"),
            },
            _ => Err("range expects integer argument"),
        },
        2 => {
            // range(start, end) -> [start, start+1, ..., end-1]
            match (&args[0], &args[1]) {
                (Value::Literal(s), Value::Literal(e)) => match (s, e) {
                    (Literal::Int(start), Literal::Int(end)) => {
                        let values: Vec<Value> = (*start..*end)
                            .map(|i| Value::Literal(Literal::Int(i)))
                            .collect();
                        Ok(Value::List(values))
                    }
                    _ => Err("range expects integer arguments"),
                },
                _ => Err("range expects integer arguments"),
            }
        }
        3 => {
            // range(start, end, step) -> [start, start+step, ..., end-step]
            match (&args[0], &args[1], &args[2]) {
                (Value::Literal(start), Value::Literal(end), Value::Literal(step)) => {
                    match (start, end, step) {
                        (Literal::Int(start), Literal::Int(end), Literal::Int(step)) => {
                            if *step == 0 {
                                return Err("range step cannot be zero");
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
                        _ => Err("range expects integer arguments"),
                    }
                }
                _ => Err("range expects integer arguments"),
            }
        }
        _ => Err("range expects 1, 2, or 3 arguments"),
    }
}
