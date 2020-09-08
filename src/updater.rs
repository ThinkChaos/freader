use actix::prelude::*;

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
        let one_hour_ago =
            chrono::Local::now().with_timezone(&chrono::Utc) - chrono::Duration::hours(1);

        let mut db = self.db.clone();
        let feed_manager = self.feed_manager.clone();

        Box::new(actix::fut::wrap_future(async move {
            log::debug!("Refreshing outdated feeds");

            let mut subscriptions =
                db.find_outdated_subscriptions(one_hour_ago)
                    .await
                    .map_err(|e| {
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
