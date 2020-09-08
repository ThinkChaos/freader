use serde::Serialize;

use super::schema::*;
use crate::db;

#[derive(Debug, Serialize, Identifiable, AsChangeset, Queryable)]
pub struct Subscription {
    pub id: db::Id,
    pub feed_url: String,
    pub title: String,
    pub site_url: Option<String>,
    pub refreshed_at: chrono::NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[table_name = "subscriptions"]
pub struct NewSubscription {
    pub feed_url: String,
    pub title: String,
    pub site_url: Option<String>,
    pub refreshed_at: chrono::NaiveDateTime,
}

impl NewSubscription {
    pub fn try_from(url: &str, feed: &feed_rs::model::Feed) -> Result<Self, &'static str> {
        let title = feed
            .title
            .as_ref()
            .map(|t| t.content.clone())
            .unwrap_or_else(|| url.to_owned());

        // Find the feed's site URL, if any
        let site_url = feed
            .links
            .iter()
            .find(|l| {
                // (rel is alternate / missing) or (media_type is html / missing)
                matches!(l.rel.as_deref(), None | Some("alternate"))
                    || matches!(l.media_type.as_deref(), None | Some("text/html"))
            })
            .map(|l| l.href.clone());

        let refreshed_at = feed
            .updated
            .as_ref()
            .unwrap_or(&chrono::Utc::now())
            .naive_utc();

        Ok(Self {
            feed_url: url.to_owned(),
            title,
            site_url,
            refreshed_at,
        })
    }
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
    pub published: chrono::NaiveDateTime,
    pub updated: chrono::NaiveDateTime,
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
    pub published: chrono::NaiveDateTime,
    pub updated: chrono::NaiveDateTime,
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
        let published = entry
            .published
            .ok_or("Missing publishing date")?
            .naive_utc();
        let updated = entry.updated.map(|d| d.naive_utc()).unwrap_or(published);
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
            published,
            updated,
            content,
            is_read: false,
            is_starred: false,
        })
    }
}
