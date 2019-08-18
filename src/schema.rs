table! {
    _subscriptions_old (id) {
        id -> Text,
        feed_url -> Text,
    }
}

table! {
    subscriptions (id) {
        id -> Text,
        feed_url -> Text,
        title -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    _subscriptions_old,
    subscriptions,
);
