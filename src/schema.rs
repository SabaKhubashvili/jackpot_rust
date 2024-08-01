// @generated automatically by Diesel CLI.

diesel::table! {
    jackpotgames (game_id) {
        game_id -> Int4,
        start_time -> Nullable<Timestamp>,
        end_time -> Nullable<Timestamp>,
        #[max_length = 20]
        status -> Nullable<Varchar>,
        winner_id -> Nullable<Int4>,
    }
}

diesel::table! {
    jackpotplayers (player_id) {
        player_id -> Int4,
        amount -> Nullable<Float8>,
        session_id -> Nullable<Int4>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 255]
        username -> Varchar,
        #[max_length = 255]
        hashed_password -> Varchar,
        balance -> Int4,
        created_at -> Timestamp,
    }
}

diesel::joinable!(jackpotplayers -> users (player_id));

diesel::allow_tables_to_appear_in_same_query!(
    jackpotgames,
    jackpotplayers,
    users,
);
