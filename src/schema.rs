table! {
    _subscriptions_old (id) {
        id -> Binary,
        feed_url -> Text,
    }
}

table! {
    subscriptions (id) {
        id -> Binary,
        feed_url -> Text,
        title -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    _subscriptions_old,
    subscriptions,
);
