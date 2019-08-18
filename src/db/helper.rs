use actix::prelude::*;
use actix_web::ResponseError;
use futures::future::{self, Future};
use std::fmt::{self, Display};

use crate::models::*;
use super::executor::*;


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


pub trait DatabaseFuture<I>: Future<Item=I, Error = Error> {}
impl<I, T: Future<Item=I, Error = Error>> DatabaseFuture<I> for T {}


#[derive(Clone)]
pub struct Helper {
    executor: Addr<Executor>,
}

impl Helper {
    pub fn new(executor: Addr<Executor>) -> Self {
        Helper { executor }
    }

    fn map<F, M>(&mut self, future: F) -> impl DatabaseFuture<M>
    where
        F: Future<Item = Result<M, diesel::result::Error>, Error = MailboxError>,
    {
        future
            .map_err(|e| Error::MailboxError(e))
            .and_then(|r| match r {
                Ok(r) => future::ok(r),
                Err(e) => future::err(Error::DatabaseError(e)),
            })
    }

    pub fn create_subscription(&mut self, feed_url: String) -> impl DatabaseFuture<Subscription> {
        self.map(self.executor.send(
            CreateSubscription { feed_url: feed_url.clone(), title: feed_url }
        ))
    }

    pub fn get_subscriptions(&mut self) -> impl DatabaseFuture<Vec<Subscription>> {
        self.map(self.executor.send(
            GetSubscriptions
        ))
    }
}
