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

#[derive(Debug, Serialize, Identifiable, AsChangeset, Queryable)]
pub struct Item {
    pub id: db::Id,
    pub subscription_id: db::Id,
    pub url: String,
    pub title: String,
    pub author: String,
    pub content: String,
    pub is_read: bool,
    pub is_starred: bool,
}

#[derive(Debug, Insertable)]
#[table_name = "items"]
pub struct NewItem {
    pub subscription_id: db::Id,
    pub url: String,
    pub title: String,
    pub author: String,
    pub content: String,
    pub is_read: bool,
    pub is_starred: bool,
}

impl NewItem {
    pub fn try_from(
        entry: &feed_rs::model::Entry,
        subscription: &Subscription,
    ) -> Result<Self, &'static str> {
        let url = entry.links.first().ok_or("Missing URL")?.href.clone();
        let title = entry.title.as_ref().ok_or("Missing title")?.content.clone();
        let author = entry.authors.first().ok_or("Missing author")?.name.clone();
        let content = entry
            .content
            .as_ref()
            .and_then(|c| c.body.as_ref())
            .ok_or("Missing content")?
            .clone();

        Ok(Self {
            subscription_id: subscription.id,
            url,
            title,
            author,
            content,
            is_read: false,
            is_starred: false,
        })
    }
}
