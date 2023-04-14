// @generated automatically by Diesel CLI.

diesel::table! {
    use crate::sqlite_mapping::*;

    users (username) {
        username -> Text,
        created_at -> Integer,
    }
}
