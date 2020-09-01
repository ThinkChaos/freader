use actix::prelude::*;
use actix_web::ResponseError;
use futures::future::{self, TryFutureExt};
use std::fmt::{self, Display};
use std::future::Future;

use super::{executor::*, models::*, Id};

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
    pub fn new(executor: Addr<Executor>) -> Self {
        Helper { executor }
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

    pub fn create_subscription(
        &mut self,
        feed_url: String,
        title: String,
        site_url: Option<String>,
    ) -> impl DatabaseFuture<Subscription> {
        Self::map(self.executor.send(CreateSubscription {
            feed_url,
            title,
            site_url,
        }))
    }

    pub fn get_subscription(&mut self, id: Id) -> impl DatabaseFuture<Subscription> {
        Self::map(self.executor.send(GetSubscription(id)))
    }

    pub fn get_subscriptions(&mut self) -> impl DatabaseFuture<Vec<Subscription>> {
        Self::map(self.executor.send(GetSubscriptions))
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

    pub fn get_items_and_subscriptions(
        &mut self,
        ids: Vec<Id>,
    ) -> impl DatabaseFuture<Vec<(Item, Subscription)>> {
        Self::map(self.executor.send(GetItemsAndSubscriptions(ids)))
    }

    pub fn find_items(
        &mut self,
        read: Option<bool>,
        starred: Option<bool>,
        max_items: usize,
    ) -> impl DatabaseFuture<Vec<Item>> {
        Self::map(self.executor.send(FindItems {
            read,
            starred,
            max_items,
        }))
    }
}
