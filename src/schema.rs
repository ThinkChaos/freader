table! {
    categories (id) {
        id -> Integer,
        name -> Text,
    }
}

table! {
    subscription_categories (subscription_id, category_id) {
        subscription_id -> Integer,
        category_id -> Integer,
    }
}

table! {
    subscriptions (id) {
        id -> Integer,
        feed_url -> Text,
        title -> Text,
        site_url -> Nullable<Text>,
    }
}

joinable!(subscription_categories -> categories (category_id));
joinable!(subscription_categories -> subscriptions (subscription_id));

allow_tables_to_appear_in_same_query!(
    categories,
    subscription_categories,
    subscriptions,
);
