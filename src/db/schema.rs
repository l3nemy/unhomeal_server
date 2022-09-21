// @generated automatically by Diesel CLI.

diesel::table! {
    applications (id) {
        id -> Unsigned<Bigint>,
        user_id -> Unsigned<Bigint>,
        created_at -> Datetime,
    }
}

diesel::table! {
    meals (id) {
        id -> Unsigned<Bigint>,
        name -> Varchar,
        date -> Date,
    }
}

diesel::table! {
    rates (id) {
        id -> Unsigned<Bigint>,
        user_id -> Unsigned<Bigint>,
        food_name -> Varchar,
        rate_level -> Tinyint,
        created_at -> Datetime,
    }
}

diesel::table! {
    total_rates (id) {
        id -> Unsigned<Bigint>,
        user_id -> Unsigned<Bigint>,
        rate_level -> Unsigned<Tinyint>,
        created_at -> Datetime,
    }
}

diesel::table! {
    users (id) {
        id -> Unsigned<Bigint>,
        username -> Varchar,
        name -> Varchar,
        session_id -> Nullable<Varchar>,
        auto_apply -> Bool,
        is_teacher -> Bool,
        created_at -> Datetime,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    applications,
    meals,
    rates,
    total_rates,
    users,
);
