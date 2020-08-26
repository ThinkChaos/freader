table! {
    _subscriptions_old (id) {
        id -> Binary,
        feed_url -> Text,
    }
}

table! {
    categories (id) {
        id -> Binary,
        name -> Text,
    }
}

table! {
    subscription_categories (subscription_id, category_id) {
        subscription_id -> Binary,
        category_id -> Binary,
    }
}

table! {
    subscriptions (id) {
        id -> Binary,
        feed_url -> Text,
        title -> Text,
        site_url -> Nullable<Text>,
    }
}

joinable!(subscription_categories -> categories (category_id));
joinable!(subscription_categories -> subscriptions (subscription_id));

allow_tables_to_appear_in_same_query!(
    _subscriptions_old,
    categories,
    subscription_categories,
    subscriptions,
);
