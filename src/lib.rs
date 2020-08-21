#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unknown_lints)]

#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate serde;

pub use value::to_value;
pub use value::Value;

mod error;
mod path;
mod value;

#[cfg(test)]
mod tests {
    use crate::{to_value, Value};

    #[test]
    fn simple_test() {
        let mut value_origin = Value::default();

        assert!(matches!(value_origin.set("/test/bool", true), Ok(_)));
        assert!(matches!(
            value_origin.set("/test/str", "i am string"),
            Ok(_)
        ));

        assert!(matches!(value_origin.get("/test/bool"), Ok(Some(true))));
        assert!(
            matches!(value_origin.get::<String, _, _>("/test/str"), Ok(Some(str)) if str == "i am string")
        );

        let mut value_new = Value::default();

        assert!(matches!(value_new.set("/test/bool", false), Ok(_)));
        assert!(matches!(value_new.set("/test/i32", 1000_i32), Ok(_)));

        assert!(matches!(value_origin.merge(value_new), Ok(_)));

        assert!(matches!(value_origin.get("/test/bool"), Ok(Some(false))));
        assert!(
            matches!(value_origin.get::<String, _, _>("/test/str"), Ok(Some(str)) if str == "i am string")
        );
        assert!(matches!(value_origin.get("/test/i32"), Ok(Some(1000_i32))));
    }

    #[test]
    fn simple_to_value_test() {
        let bool_value = true;
        assert!(
            matches!(to_value(bool_value), Ok(v) if matches!(v.get::<bool, _, _>("/fake_path"), Ok(None)))
        );
        let i32_value = 1000_i32;
        assert!(
            matches!(to_value(i32_value), Ok(v) if matches!(v.get::<i32, _, _>("/fake_path"), Ok(None)))
        );
        let str_value = "i am string";
        assert!(
            matches!(to_value(str_value), Ok(v) if matches!(v.get::<String, _, _>("/fake_path"), Ok(None)))
        );

        let value = to_value(str_value);
        assert!(matches!(to_value(str_value), Ok(_)));

        let mut value = value.unwrap();

        // should override origin value inside the value
        assert!(matches!(value.set("/test/bool", false), Ok(_)));
        assert!(matches!(value.get("/test/bool"), Ok(Some(false))));
    }
}
