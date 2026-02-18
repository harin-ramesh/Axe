use axe::{Axe, Context, EvalSignal, Literal, Parser, Value};

// Helper function to parse and evaluate code
fn eval(code: &str) -> Result<(Value, Context), EvalSignal> {
    let context = Context::new();
    let mut parser = Parser::new(code, &context);
    let program = parser
        .parse()
        .map_err(|e| EvalSignal::Error(e.to_string()))?;
    let mut axe = Axe::new(&context);
    let value = axe.run(program)?;
    Ok((value, context))
}

// Helper to get int value from evaluation
fn eval_int(code: &str) -> i64 {
    match eval(code) {
        Ok((Value::Literal(Literal::Int(n)), _)) => n,
        Ok((other, _)) => panic!("Expected Int, got {:?}", other),
        Err(e) => panic!("Eval failed: {:?}", e),
    }
}

// Helper to get string value from evaluation
fn eval_str(code: &str) -> String {
    match eval(code) {
        Ok((Value::Literal(Literal::Str(s)), ctx)) => ctx.resolve(s),
        Ok((other, _)) => panic!("Expected Str, got {:?}", other),
        Err(e) => panic!("Eval failed: {:?}", e),
    }
}

// ============================================================================
// Class Definition Tests
// ============================================================================

#[test]
fn class_definition_basic() {
    let code = r#"
        class Point {
            let x = 0;
            let y = 0;
        }
    "#;
    assert!(eval(code).is_ok());
}

#[test]
fn class_definition_with_method() {
    let code = r#"
        class Counter {
            let count = 0;
            
            fn increment(self) {
                self.count = self.count + 1;
            }
        }
    "#;
    assert!(eval(code).is_ok());
}

#[test]
fn class_with_init_constructor() {
    let code = r#"
        class Person {
            let name = "";
            let age = 0;
            
            fn init(self, n, a) {
                self.name = n;
                self.age = a;
            }
        }
    "#;
    assert!(eval(code).is_ok());
}

// ============================================================================
// Object Instantiation Tests
// ============================================================================

#[test]
fn object_instantiation_basic() {
    let code = r#"
        class Box {
            let value = 0;
            
            fn init(self, v) {
                self.value = v;
            }
        }
        
        let b = new Box(42);
        b.value;
    "#;
    assert_eq!(eval_int(code), 42);
}

#[test]
fn object_instantiation_multiple_properties() {
    let code = r#"
        class Point {
            let x = 0;
            let y = 0;
            
            fn init(self, px, py) {
                self.x = px;
                self.y = py;
            }
        }
        
        let p = new Point(10, 20);
        p.x + p.y;
    "#;
    assert_eq!(eval_int(code), 30);
}

#[test]
fn object_instantiation_string_property() {
    let code = r#"
        class Greeting {
            let message = "";
            
            fn init(self, msg) {
                self.message = msg;
            }
        }
        
        let g = new Greeting("Hello, World!");
        g.message;
    "#;
    assert_eq!(eval_str(code), "Hello, World!");
}

// ============================================================================
// Method Call Tests
// ============================================================================

#[test]
fn method_call_simple() {
    let code = r#"
        class Calculator {
            let result = 0;
            
            fn init(self, initial) {
                self.result = initial;
            }
            
            fn double(self) {
                return self.result * 2;
            }
        }
        
        let calc = new Calculator(21);
        calc.double();
    "#;
    assert_eq!(eval_int(code), 42);
}

#[test]
fn method_call_with_arguments() {
    let code = r#"
        class Math {
            let base = 0;
            
            fn init(self, b) {
                self.base = b;
            }
            
            fn add(self, n) {
                return self.base + n;
            }
            
            fn multiply(self, n) {
                return self.base * n;
            }
        }
        
        let m = new Math(10);
        m.add(5) + m.multiply(3);
    "#;
    // (10 + 5) + (10 * 3) = 15 + 30 = 45
    assert_eq!(eval_int(code), 45);
}

#[test]
fn method_call_modifies_state() {
    let code = r#"
        class Counter {
            let count = 0;
            
            fn init(self, start) {
                self.count = start;
            }
            
            fn increment(self) {
                self.count = self.count + 1;
                return self.count;
            }
            
            fn get(self) {
                return self.count;
            }
        }
        
        let c = new Counter(0);
        c.increment();
        c.increment();
        c.increment();
        c.get();
    "#;
    assert_eq!(eval_int(code), 3);
}

// ============================================================================
// Property Access Tests
// ============================================================================

#[test]
fn property_access_basic() {
    let code = r#"
        class Data {
            let value = 100;
            
            fn init(self, v) {
                self.value = v;
            }
        }
        
        let d = new Data(42);
        d.value;
    "#;
    assert_eq!(eval_int(code), 42);
}

#[test]
fn property_access_nested_expression() {
    let code = r#"
        class Pair {
            let first = 0;
            let second = 0;
            
            fn init(self, a, b) {
                self.first = a;
                self.second = b;
            }
        }
        
        let p = new Pair(3, 4);
        p.first * p.first + p.second * p.second;
    "#;
    // 3^2 + 4^2 = 9 + 16 = 25
    assert_eq!(eval_int(code), 25);
}

// ============================================================================
// Inheritance Tests
// ============================================================================

#[test]
fn class_inheritance_basic() {
    let code = r#"
        class Animal {
            let name = "";
            
            fn speak(self) {
                "sound";
            }
        }
        
        class Dog : Animal {
            fn bark(self) {
                "woof";
            }
        }
    "#;
    assert!(eval(code).is_ok());
}

#[test]
fn inheritance_access_parent_property() {
    let code = r#"
        class Base {
            let value = 100;
        }
        
        class Derived : Base {
            let extra = 50;
        }
    "#;
    assert!(eval(code).is_ok());
}

// ============================================================================
// Multiple Objects Tests
// ============================================================================

#[test]
fn multiple_objects_independent() {
    let code = r#"
        class Box {
            let value = 0;
            
            fn init(self, v) {
                self.value = v;
            }
            
            fn get(self) {
                return self.value;
            }
        }
        
        let a = new Box(10);
        let b = new Box(20);
        let c = new Box(30);
        
        a.get() + b.get() + c.get();
    "#;
    assert_eq!(eval_int(code), 60);
}

// ============================================================================
// Class with Complex Methods Tests
// ============================================================================

#[test]
fn class_with_conditional_method() {
    let code = r#"
        class Number {
            let value = 0;
            
            fn init(self, v) {
                self.value = v;
            }
            
            fn isPositive(self) {
                if (self.value > 0) {
                    return true;
                } else {
                    return false;
                }
            }
        }
        
        let n = new Number(5);
        n.isPositive();
    "#;
    match eval(code) {
        Ok((Value::Literal(Literal::Bool(b)), _)) => assert!(b),
        other => panic!("Expected Bool(true), got {:?}", other),
    }
}

#[test]
fn class_with_loop_in_method() {
    let code = r#"
        class Summer {
            let limit = 0;
            
            fn init(self, n) {
                self.limit = n;
            }
            
            fn sum(self) {
                let total = 0;
                let i = 1;
                while (i <= self.limit) {
                    total = total + i;
                    i = i + 1;
                }
                return total;
            }
        }
        
        let s = new Summer(10);
        s.sum();
    "#;
    assert_eq!(eval_int(code), 55);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn class_not_found_error() {
    let code = r#"
        let x = new NonExistent(1);
    "#;
    assert!(eval(code).is_err());
}

#[test]
fn method_not_found_error() {
    let code = r#"
        class Empty {
            fn init(self) {}
        }
        
        let e = new Empty();
        e.nonexistent();
    "#;
    assert!(eval(code).is_err());
}

#[test]
fn property_not_found_error() {
    let code = r#"
        class Simple {
            let x = 1;
            fn init(self) {}
        }
        
        let s = new Simple();
        s.y;
    "#;
    assert!(eval(code).is_err());
}

// ============================================================================
// Static Access (::) Tests
// ============================================================================

#[test]
fn static_property_access() {
    let code = r#"
        class Config {
            let max = 100;
        }
        
        Config::max;
    "#;
    assert_eq!(eval_int(code), 100);
}

#[test]
fn static_property_access_string() {
    let code = r#"
        class App {
            let name = "axe";
        }
        
        App::name;
    "#;
    assert_eq!(eval_str(code), "axe");
}

#[test]
fn static_method_call_no_args() {
    let code = r#"
        class MathUtils {
            fn pi() {
                return 3;
            }
        }
        
        MathUtils::pi();
    "#;
    assert_eq!(eval_int(code), 3);
}

#[test]
fn static_method_call_with_args() {
    let code = r#"
        class MathUtils {
            fn add(a, b) {
                return a + b;
            }
        }
        
        MathUtils::add(10, 20);
    "#;
    assert_eq!(eval_int(code), 30);
}

#[test]
fn static_property_not_found_error() {
    let code = r#"
        class Empty {}
        
        Empty::missing;
    "#;
    assert!(eval(code).is_err());
}

#[test]
fn static_method_not_found_error() {
    let code = r#"
        class Empty {}
        
        Empty::missing();
    "#;
    assert!(eval(code).is_err());
}

#[test]
fn static_access_on_non_class_error() {
    let code = r#"
        let x = 42;
        x::foo;
    "#;
    assert!(eval(code).is_err());
}

#[test]
fn static_and_instance_coexist() {
    let code = r#"
        class Counter {
            let default_start = 0;
            
            fn init(self, n) {
                self.count = n;
            }
            
            fn get(self) {
                return self.count;
            }
        }
        
        let c = new Counter(5);
        Counter::default_start + c.get();
    "#;
    // 0 + 5 = 5
    assert_eq!(eval_int(code), 5);
}

#[test]
fn static_method_returns_value_used_in_expression() {
    let code = r#"
        class Factory {
            fn magic() {
                return 42;
            }
        }
        
        Factory::magic() + 8;
    "#;
    assert_eq!(eval_int(code), 50);
}

#[test]
fn static_property_multiple_classes() {
    let code = r#"
        class A {
            let val = 10;
        }
        
        class B {
            let val = 20;
        }
        
        A::val + B::val;
    "#;
    assert_eq!(eval_int(code), 30);
}
