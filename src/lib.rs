#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unknown_lints)]

#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate serde;

pub use value::Value;

mod error;
mod path;
mod value;

#[cfg(test)]
mod tests {
    use crate::Value;

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
}
