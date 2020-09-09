use actix::prelude::*;
use actix_web::ResponseError;
use diesel::prelude::*;
use futures::future::{self, TryFutureExt};
use std::fmt::{self, Display};
use std::future::Future;

use super::{executor::*, models::*, schema, Id};
use crate::config::Config;

#[derive(Debug)]
pub enum Error {
    MailboxError(MailboxError),
    DatabaseError(diesel::result::Error),
}

impl ResponseError for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::MailboxError(e) => write!(f, "MailboxError: {}", e),
            Self::DatabaseError(e) => write!(f, "DatabaseError: {}", e),
        }
    }
}


pub trait DatabaseFuture<I>: Future<Output = Result<I, Error>> {}
impl<I, T: Future<Output = Result<I, Error>>> DatabaseFuture<I> for T {}


#[derive(Clone)]
pub struct Helper {
    executor: Addr<Executor>,
}

impl Helper {
    pub fn new(cfg: &Config) -> diesel::result::ConnectionResult<Self> {
        // Test DB connection now
        drop(Executor::connect(&cfg.sqlite_db)?);

        let sqlite_db = cfg.sqlite_db.clone();
        let executor = SyncArbiter::start(2, move || {
            Executor::connect(&sqlite_db).expect("DB connection failed")
        });

        Ok(Helper { executor })
    }

    fn map<F, M>(future: F) -> impl DatabaseFuture<M>
    where
        F: Future<Output = Result<diesel::QueryResult<M>, MailboxError>>,
    {
        future.map_err(Error::MailboxError).and_then(|r| match r {
            Ok(r) => future::ok(r),
            Err(e) => future::err(Error::DatabaseError(e)),
        })
    }

    fn find_all<F, Q, T>(&mut self, query_builder: F) -> impl DatabaseFuture<Vec<T>>
    where
        F: 'static + FnOnce() -> Q + Send,
        Q: 'static + diesel::query_dsl::LoadQuery<diesel::SqliteConnection, T>,
        T: 'static + Send,
    {
        Self::map(self.executor.send(FindAll::new(query_builder)))
    }

    pub fn create_subscription(
        &mut self,
        new_subscription: NewSubscription,
    ) -> impl DatabaseFuture<Subscription> {
        Self::map(self.executor.send(CreateSubscription(new_subscription)))
    }

    pub fn remove_subscription(&mut self, id: Id) -> impl DatabaseFuture<()> {
        Self::map(self.executor.send(RemoveSubscription(id)))
    }

    pub fn get_subscription(&mut self, id: Id) -> impl DatabaseFuture<Subscription> {
        Self::map(self.executor.send(GetSubscription(id)))
    }

    pub fn get_subscriptions(&mut self) -> impl DatabaseFuture<Vec<Subscription>> {
        self.find_all(|| {
            use schema::subscriptions::dsl::*;

            subscriptions
        })
    }

    pub fn find_outdated_subscriptions(
        &mut self,
        updated_before: chrono::DateTime<chrono::Utc>,
    ) -> impl DatabaseFuture<Vec<Subscription>> {
        self.find_all(move || {
            use schema::subscriptions::dsl::*;

            let updated_before = updated_before.naive_utc();

            subscriptions.filter(refreshed_at.le(updated_before))
        })
    }

    pub fn update_subscription(
        &mut self,
        subscription: Subscription,
    ) -> impl DatabaseFuture<Subscription> {
        Self::map(self.executor.send(UpdateSubscription(subscription)))
    }

    pub fn transform_subscription<F>(
        &mut self,
        id: Id,
        transform: F,
    ) -> impl DatabaseFuture<Subscription>
    where
        F: FnOnce(&mut Subscription) + Send + 'static,
    {
        Self::map(
            self.executor
                .send(TransformSubscription(id, Box::new(transform))),
        )
    }

    pub fn subscription_add_category(
        &mut self,
        subscription_id: Id,
        category: String,
    ) -> impl DatabaseFuture<Category> {
        Self::map(self.executor.send(SubscriptionAddCategory {
            subscription_id,
            category_name: category,
        }))
    }

    pub fn subscription_remove_category(
        &mut self,
        subscription_id: Id,
        category: String,
    ) -> impl DatabaseFuture<()> {
        Self::map(self.executor.send(SubscriptionRemoveCategory {
            subscription_id,
            category_name: category,
        }))
    }

    pub fn get_subscription_categories(
        &mut self,
        subscription_id: Id,
    ) -> impl DatabaseFuture<Vec<Category>> {
        Self::map(
            self.executor
                .send(GetSubscriptionCategories(subscription_id)),
        )
    }

    pub fn create_item(&mut self, new_item: NewItem) -> impl DatabaseFuture<Item> {
        Self::map(self.executor.send(CreateItem(new_item)))
    }

    pub fn get_item(&mut self, id: Id) -> impl DatabaseFuture<Item> {
        Self::map(self.executor.send(GetItem(id)))
    }

    pub fn get_items_and_subscriptions(
        &mut self,
        item_ids: Vec<Id>,
    ) -> impl DatabaseFuture<Vec<(Item, Subscription)>> {
        self.find_all(move || {
            use schema::items::dsl::*;

            items
                .filter(id.eq_any(item_ids))
                .inner_join(schema::subscriptions::table)
        })
    }

    pub fn update_item(&mut self, item: Item) -> impl DatabaseFuture<Item> {
        Self::map(self.executor.send(UpdateItem(item)))
    }

    pub fn find_items(
        &mut self,
        read: Option<bool>,
        starred: Option<bool>,
        max_items: usize,
    ) -> impl DatabaseFuture<Vec<Item>> {
        self.find_all(move || {
            use schema::items::dsl::*;

            let mut query = items.into_boxed();

            if let Some(val) = read {
                query = query.filter(is_read.eq(val));
            }

            if let Some(val) = starred {
                query = query.filter(is_starred.eq(val));
            }

            query.limit(max_items as i64)
        })
    }

    pub fn get_subscription_items(
        &mut self,
        subscription_id_: Id,
    ) -> impl DatabaseFuture<Vec<Item>> {
        self.find_all(move || {
            use schema::items::dsl::*;
            items.filter(subscription_id.eq(subscription_id_))
        })
    }
}
