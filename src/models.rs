use serde::Serialize;

use super::schema::subscriptions;


#[derive(Debug, Serialize, Queryable)]
pub struct Subscription {
    pub id: String,
    pub feed_url: String,
    pub title: String,
}

#[derive(Debug, Insertable)]
#[table_name = "subscriptions"]
pub struct NewSubscription<'a> {
    pub id: &'a str,
    pub feed_url: &'a str,
    pub title: &'a str,
}
