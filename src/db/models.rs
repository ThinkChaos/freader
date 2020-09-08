use serde::Serialize;

use super::schema::*;
use crate::db;
use crate::utils::make_url_absolute;

#[derive(Debug, Clone, Serialize, Identifiable, AsChangeset, Queryable)]
pub struct Subscription {
    pub id: db::Id,
    pub feed_url: String,
    pub title: String,
    pub site_url: Option<String>,
    pub refreshed_at: chrono::NaiveDateTime,
}

impl std::fmt::Display for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.title, self.feed_url)
    }
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
    pub fn try_from(feed_url: &str, feed: &feed_rs::model::Feed) -> Result<Self, &'static str> {
        let title = feed
            .title
            .as_ref()
            .map(|t| t.content.clone())
            .unwrap_or_else(|| feed_url.to_owned());

        // Find the feed's site URL, if any
        let site_url = feed
            .links
            .iter()
            .find(|l| {
                // (rel is alternate / missing) or (media_type is html / missing)
                matches!(l.rel.as_deref(), None | Some("alternate"))
                    || matches!(l.media_type.as_deref(), None | Some("text/html"))
            })
            .and_then(|l| make_url_absolute(&l.href, &feed_url).ok());

        let refreshed_at = feed
            .updated
            .as_ref()
            .unwrap_or(&chrono::Utc::now())
            .naive_utc();

        Ok(Self {
            feed_url: feed_url.to_owned(),
            title,
            site_url,
            refreshed_at,
        })
    }
}

#[derive(Debug, Clone, Serialize, Identifiable, AsChangeset, Queryable)]
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

#[derive(Debug, Clone, Serialize, AsChangeset, Queryable)]
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

#[derive(Debug, Clone, Serialize, Identifiable, AsChangeset, Queryable)]
pub struct Item {
    pub id: db::Id,
    pub subscription_id: db::Id,
    pub url: String,
    pub title: String,
    pub author: Option<String>,
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
    pub author: Option<String>,
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
        let url = entry
            .links
            .first()
            .as_ref()
            .map(|l| &l.href)
            .or_else(|| {
                if entry.id.starts_with("http") {
                    Some(&entry.id)
                } else {
                    None
                }
            })
            .ok_or("Missing URL")?;
        let title = entry.title.as_ref().ok_or("Missing title")?.content.clone();
        let author = entry.authors.first().map(|a| a.name.clone());
        let published = entry
            .published
            .or(entry.updated)
            .unwrap_or_else(|| chrono::Local::now().with_timezone(&chrono::Utc))
            .naive_utc();
        let updated = entry.updated.map(|d| d.naive_utc()).unwrap_or(published);
        let content = entry
            .content
            .as_ref()
            .and_then(|c| c.body.as_ref())
            .or_else(|| entry.summary.as_ref().map(|s| &s.content))
            .map(String::clone)
            .unwrap_or_default();

        let url = make_url_absolute(url, &subscription.feed_url)?;

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
