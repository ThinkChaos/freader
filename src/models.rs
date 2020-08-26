use serde::Serialize;

use super::schema::*;
use crate::db;

#[derive(Debug, Serialize, Identifiable, AsChangeset, Queryable)]
pub struct Subscription {
    pub id: db::Id,
    pub feed_url: String,
    pub title: String,
    pub site_url: Option<String>,
}

#[derive(Debug, Insertable)]
#[table_name = "subscriptions"]
pub struct NewSubscription<'a> {
    pub feed_url: &'a str,
    pub title: &'a str,
    pub site_url: Option<&'a str>,
}

#[derive(Debug, Serialize, Identifiable, AsChangeset, Queryable)]
#[table_name = "categories"]
pub struct Category {
    pub id: db::Id,
    pub name: String,
}

#[derive(Debug, Insertable)]
#[table_name = "categories"]
pub struct NewCategory<'a> {
    pub name: &'a str,
}

#[derive(Debug, Serialize, AsChangeset, Queryable)]
#[table_name = "subscription_categories"]
pub struct SubscriptionCategory {
    pub subscription_id: db::Id,
    pub category_id: db::Id,
}

#[derive(Debug, Insertable)]
#[table_name = "subscription_categories"]
pub struct NewSubscriptionCategory<'a> {
    pub subscription_id: &'a db::Id,
    pub category_id: &'a db::Id,
}
