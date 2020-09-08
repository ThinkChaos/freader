table! {
    categories (id) {
        id -> Integer,
        name -> Text,
    }
}

table! {
    items (id) {
        id -> Integer,
        subscription_id -> Integer,
        url -> Text,
        title -> Text,
        author -> Text,
        published -> Timestamp,
        updated -> Timestamp,
        content -> Text,
        is_read -> Bool,
        is_starred -> Bool,
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
        refreshed_at -> Timestamp,
    }
}

joinable!(items -> subscriptions (subscription_id));
joinable!(subscription_categories -> categories (category_id));
joinable!(subscription_categories -> subscriptions (subscription_id));

allow_tables_to_appear_in_same_query!(
    categories,
    items,
    subscription_categories,
    subscriptions,
);
