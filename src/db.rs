use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::*;

pub struct Executor {
    conn: SqliteConnection,
}

impl Executor {
    pub fn new(connspec: &str) -> ConnectionResult<Executor> {
        Ok(Executor {
            conn: SqliteConnection::establish(connspec)?
        })
    }
}

impl Actor for Executor {
    type Context = SyncContext<Self>;
}


pub struct CreateSubscription {
    pub feed_url: String,
}

impl Message for CreateSubscription {
    type Result = diesel::QueryResult<Subscription>;
}

impl Handler<CreateSubscription> for Executor {
    type Result = <CreateSubscription as Message>::Result;

    fn handle(&mut self, msg: CreateSubscription, _: &mut Self::Context) -> Self::Result {
        use crate::schema::subscriptions::dsl::*;

        let uuid = Uuid::new_v4().to_string();
        let subscription = NewSubscription {
            id: &uuid,
            feed_url: &msg.feed_url,
        };

        diesel::insert_into(subscriptions)
            .values(&subscription)
            .execute(&self.conn)?;

        let mut items = subscriptions
            .filter(id.eq(&uuid))
            .load(&self.conn)?;

        Ok(items.pop().unwrap())
    }
}


pub struct GetSubscriptions;

impl Message for GetSubscriptions {
    type Result = diesel::QueryResult<Vec<Subscription>>;
}

impl Handler<GetSubscriptions> for Executor {
    type Result = <GetSubscriptions as Message>::Result;

    fn handle(&mut self, _: GetSubscriptions, _: &mut Self::Context) -> Self::Result {
        use crate::schema::subscriptions::dsl::*;

        subscriptions.load(&self.conn)
    }
}
