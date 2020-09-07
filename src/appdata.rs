use crate::feed_manager::FeedManager;
use crate::prelude::*;

pub struct AppData {
    pub cfg: Config,
    pub db: db::Helper,
    pub feed_manager: FeedManager,
}

impl AppData {
    pub fn new(cfg: Config, db: db::Helper, feed_manager: FeedManager) -> Self {
        AppData {
            cfg,
            db,
            feed_manager,
        }
    }
}
