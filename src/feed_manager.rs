use feed_rs::model::{Entry, Feed};
use futures::TryFutureExt;

use crate::db::models::{NewItem, NewSubscription, Subscription};
use crate::prelude::*;
use crate::updater::Updater;

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

        self.store_new_entries(&subscription, feed.entries).await?;

        Ok(subscription)
    }

    /// Fetch feed and store new items.
    ///
    /// The subscription is updated with a new `next_refresh`. This
    /// change is reflected in the database.
    ///
    /// Result is the number of new items.
    pub async fn refresh(&self, subscription: &mut Subscription) -> Result<usize, &'static str> {
        let feed = self.fetch(&subscription.feed_url).await?;

        let count = self.store_new_entries(&subscription, feed.entries).await?;

        // Update the subscription's refresh time
        let mut db = self.db.clone();
        subscription.next_refresh = Updater::next_refresh().naive_utc();
        *subscription = db
            .update_subscription(subscription.clone())
            .await
            .map_err(|e| {
                log::error!("Could not update subscription in db: {}", e);
                "Database error."
            })?;

        Ok(count)
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

    async fn store_new_entries(
        &self,
        subscription: &Subscription,
        entries: Vec<Entry>,
    ) -> Result<usize, &'static str> {
        let mut db = self.db.clone();

        let existing = db.get_subscription_items(subscription.id).await.unwrap();
        let existing = existing
            .iter()
            .map(|item| (item.title.as_str(), item.url.as_str()))
            .collect::<std::collections::HashSet<_>>();

        let mut any_ok = false;
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

            any_ok = true;

            if existing.contains(&(new_item.title.as_str(), new_item.url.as_str())) {
                log::trace!("Ignoring existing item: {}", new_item.title);
                continue;
            }

            count += 1;
            db.create_item(new_item).await.map_err(|e| {
                log::error!("Could not store item: {}", e);
                "Database error."
            })?;
        }

        if any_ok || entries.is_empty() {
            Ok(count)
        } else {
            Err("No entry could be parsed.")
        }
    }
}
