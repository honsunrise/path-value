# path-value

![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)
[![Build Status](https://github.com/hosunrise/path-value/workflows/CI/badge.svg)](https://github.com/hosunrise/path-value/actions?query=workflow%3ACI+)
[![Crate Status](https://img.shields.io/crates/v/path-value.svg)](https://crates.io/crates/path-value)
[![Docs Status](https://docs.rs/path-value/badge.svg)](https://docs.rs/crate/path-value/)

`path-value` is a [Rust](https://www.rust-lang.org) universal type library used to access property(s) in `Value` by path.

### Quick start

``` Rust
use path_value::Value;

fn main() {
    let mut value_origin = Value::default();

    value_origin.set("/test/bool", true).unwrap();
    value_origin.set("/test/str", "i am string").unwrap();

    println!("{}", value_origin.get::<bool, _, _>("/test/bool").unwrap().unwrap());
    println!("{}", value_origin.get::<String, _, _>("/test/str").unwrap().unwrap());

    println!("\nAfter merge\n");

    let mut value_new = Value::default();

    value_new.set("/test/bool", false).unwrap();
    value_new.set("/test/i32", 1000_i32).unwrap();

    value_origin.merge(value_new).unwrap();

    println!("{}", value_origin.get::<bool, _, _>("/test/bool").unwrap().unwrap());
    println!("{}", value_origin.get::<String, _, _>("/test/str").unwrap().unwrap());
    println!("{}", value_origin.get::<i32, _, _>("/test/i32").unwrap().unwrap());
}
```

### [Documentation](https://docs.rs/path-value)

## License

[MIT](LICENSE)
