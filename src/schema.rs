// @generated automatically by Diesel CLI.

diesel::table! {
    use crate::sqlite_mapping::*;

    events (id) {
        id -> Integer,
        title -> Text,
        description -> Nullable<Text>,
        color -> Text,
        start_date -> Integer,
        end_date -> Integer,
        location_lng -> Nullable<Float>,
        location_lat -> Nullable<Float>,
        location_name -> Nullable<Float>,
        created_at -> Integer,
        edited_at -> Nullable<Integer>,
    }
}

diesel::table! {
    use crate::sqlite_mapping::*;

    users (username) {
        username -> Text,
        created_at -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    events,
    users,
);
