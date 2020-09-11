use actix::prelude::*;
use rand::Rng;

use crate::db::models::Subscription;
use crate::feed_manager::FeedManager;
use crate::prelude::*;

/// Actor that periodically refreshes subscriptions.
pub struct Updater {
    db: db::Helper,
    feed_manager: FeedManager,
}

impl Updater {
    pub fn new(db: db::Helper, feed_manager: FeedManager) -> Self {
        Updater { db, feed_manager }
    }

    /// Generate a DateTime one hour from (with a small random offset).
    ///
    /// We want to refresh each once per hour. To avoid refreshing all
    /// feeds at once all the time, we add a small delay.
    /// Thus even if all feeds start being refreshed at once, they will
    /// progressively be refreshed separately.
    ///
    /// We also take into account how many consecutive refresh errors occurred,
    /// to avoid always refreshing a broken feed.
    pub fn next_refresh(subscription: &Subscription) -> chrono::DateTime<chrono::Local> {
        let now = chrono::Local::now();
        let base_offset = chrono::Duration::hours(1);
        let error_backoff = chrono::Duration::hours(subscription.error_count.min(16) as i64);
        let random_offset = chrono::Duration::minutes(rand::thread_rng().gen_range(-5, 5));

        now + std::cmp::max(base_offset, error_backoff) + random_offset
    }

    fn refresh_outdated(&mut self, ctx: &mut <Self as Actor>::Context) {
        ctx.notify(RefreshOutdated);
    }
}

impl Actor for Updater {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Refresh outdated subscriptions now...
        self.refresh_outdated(ctx);

        // ...and every 5 minutes
        ctx.run_interval(
            std::time::Duration::from_secs(5 * 60),
            Self::refresh_outdated,
        );
    }
}

/// Refresh subscriptions that haven't been updated in a while.
struct RefreshOutdated;

impl Message for RefreshOutdated {
    type Result = Result<(), ()>;
}

impl Handler<RefreshOutdated> for Updater {
    type Result = ResponseActFuture<Self, <RefreshOutdated as Message>::Result>;

    fn handle(&mut self, _: RefreshOutdated, _: &mut Self::Context) -> Self::Result {
        let mut db = self.db.clone();
        let feed_manager = self.feed_manager.clone();

        Box::new(actix::fut::wrap_future(async move {
            log::debug!("Refreshing outdated feeds");

            let mut subscriptions = db.find_outdated_subscriptions().await.map_err(|e| {
                log::error!("Could not load outdated subscriptions from db: {}", e);
            })?;

            let mut new_items = 0;
            let mut errors = 0;

            for mut subscription in &mut subscriptions {
                log::debug!("Fetching items for {}", subscription);
                match feed_manager.refresh(&mut subscription).await {
                    Ok(n) => new_items += n,
                    Err(e) => {
                        log::error!("Could not refresh {}: {}", subscription, e);
                        errors += 1;
                    }
                }
            }

            if !subscriptions.is_empty() {
                log::info!(
                    "Successfully refreshed {} feeds and found {} new items",
                    subscriptions.len() - errors,
                    new_items
                );
            } else {
                log::debug!("No outdated feed to refresh");
            }

            Ok(())
        }))
    }
}
