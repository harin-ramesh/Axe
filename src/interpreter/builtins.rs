//! Built-in native functions for the interpreter.

use crate::ast::Literal;

use super::tree_walker::EvalSignal;
use super::value::Value;

pub fn native_print(args: &[Value]) -> Result<Value, EvalSignal> {
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

pub fn native_type(args: &[Value]) -> Result<Value, EvalSignal> {
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
        Value::NativeFunction(..) => "native-function",
        Value::Object(..) => "object",
    };
    Ok(Value::Literal(Literal::Str(type_name.to_string())))
}

pub fn native_range(args: &[Value]) -> Result<Value, EvalSignal> {
    match args.len() {
        1 => match &args[0] {
            Value::Literal(n) => match n {
                Literal::Int(n_val) => {
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
            _ => Err("range expects integer argument".into()),
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
                    _ => Err("range expects integer arguments".into()),
                },
                _ => Err("range expects integer arguments".into()),
            }
        }
        3 => {
            // range(start, end, step) -> [start, start+step, ..., end-step]
            match (&args[0], &args[1], &args[2]) {
                (Value::Literal(start), Value::Literal(end), Value::Literal(step)) => {
                    match (start, end, step) {
                        (Literal::Int(start), Literal::Int(end), Literal::Int(step)) => {
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
                    }
                }
                _ => Err("range expects integer arguments".into()),
            }
        }
        _ => Err("range expects 1, 2, or 3 arguments".into()),
    }
}
