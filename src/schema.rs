// @generated automatically by Diesel CLI.

diesel::table! {
    use crate::sqlite_mapping::*;

    event_guests (id) {
        id -> Integer,
        event_id -> Integer,
        guest_name -> Text,
    }
}

diesel::table! {
    use crate::sqlite_mapping::*;

    events (id) {
        id -> Integer,
        title -> Text,
        description -> Nullable<Text>,
        color -> Nullable<Text>,
        start_date -> Integer,
        end_date -> Integer,
        location_lng -> Nullable<Float>,
        location_lat -> Nullable<Float>,
    }
}

diesel::table! {
    use crate::sqlite_mapping::*;

    users (username) {
        username -> Text,
        created_at -> Integer,
    }
}

diesel::joinable!(event_guests -> events (event_id));
diesel::joinable!(event_guests -> users (guest_name));

diesel::allow_tables_to_appear_in_same_query!(
    event_guests,
    events,
    users,
);
