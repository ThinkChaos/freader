use serde::Serialize;

use crate::db;
use super::schema::subscriptions;


#[derive(Debug, Serialize, Identifiable, AsChangeset, Queryable)]
pub struct Subscription {
    pub id: db::Id,
    pub feed_url: String,
    pub title: String,
}

#[derive(Debug, Insertable)]
#[table_name = "subscriptions"]
pub struct NewSubscription<'a> {
    pub id: &'a db::Id,
    pub feed_url: &'a str,
    pub title: &'a str,
}
