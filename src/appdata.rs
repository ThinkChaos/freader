use actix::prelude::*;

use crate::prelude::*;

pub struct AppData {
    pub cfg: Config,
    pub db: db::Helper,
}

impl AppData {
    pub fn new(cfg: Config) -> Result<Self, diesel::result::ConnectionError> {
        // Test DB connection now
        drop(db::Executor::connect(&cfg.sqlite_db)?);

        let sqlite_db = cfg.sqlite_db.clone();
        let db_pool = SyncArbiter::start(2, move || {
            db::Executor::connect(&sqlite_db).expect("DB connection failed")
        });

        Ok(AppData {
            cfg,
            db: db::Helper::new(db_pool),
        })
    }
}
