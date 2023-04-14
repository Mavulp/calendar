// Map all sqlite integeres to i64 because they can be that large by defintion but the diesel
// developers don't want to change the default mapping in diesel.
// https://github.com/diesel-rs/diesel/issues/852

pub use diesel::sql_types::*;

pub type Integer = BigInt;
