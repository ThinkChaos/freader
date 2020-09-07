use feed_rs::model::{Entry, Feed};
use futures::TryFutureExt;

use crate::db::models::{NewItem, NewSubscription, Subscription};
use crate::prelude::*;

#[derive(Clone)]
pub struct FeedManager {
    db: db::Helper,
    http_client: reqwest::Client,
}

impl FeedManager {
    pub fn new(db: db::Helper) -> Self {
        FeedManager {
            db,
            http_client: reqwest::Client::new(),
        }
    }

    /// Subscribe to feed and fetch items.
    pub async fn subscribe(&self, url: &str) -> Result<Subscription, &'static str> {
        let feed = self.fetch(&url).await?;

        let new_subscription = NewSubscription::try_from(&url, &feed)?;

        let subscription = self
            .db
            .clone()
            .create_subscription(new_subscription)
            .await
            .unwrap();

        self.store_entries(&subscription, feed.entries).await?;

        Ok(subscription)
    }

    async fn fetch(&self, url: &str) -> Result<Feed, &'static str> {
        let feed_bytes = self
            .http_client
            .get(url)
            .send()
            .and_then(|resp| resp.bytes())
            .await
            .map_err(|e| {
                log::error!("{}", e);
                "Could not fetch feed."
            })?;

        feed_rs::parser::parse(feed_bytes.as_ref()).map_err(|e| {
            log::error!("Parse error for {}: {}", url, e);
            "Could not parse content as a feed."
        })
    }

    async fn store_entries(
        &self,
        subscription: &Subscription,
        entries: Vec<Entry>,
    ) -> Result<usize, &'static str> {
        let mut db = self.db.clone();

        let mut count = 0;
        for entry in &entries {
            let new_item = match NewItem::try_from(entry, &subscription) {
                Ok(item) => item,
                Err(e) => {
                    log::error!("{}", e);
                    log::debug!("{:#?}", entry);
                    continue;
                }
            };

            count += 1;
            db.create_item(new_item).await.map_err(|e| {
                log::error!("{}", e);
                "Database error."
            })?;
        }

        if count > 0 || entries.is_empty() {
            Ok(count)
        } else {
            Err("No entry could be parsed.")
        }
    }
}
