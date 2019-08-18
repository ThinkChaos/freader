use actix::prelude::*;
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::*;

pub struct Executor {
    conn: SqliteConnection,
}

impl Executor {
    pub fn new(connspec: &str) -> ConnectionResult<Self> {
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
    pub title: String,
}

impl Message for CreateSubscription {
    type Result = diesel::QueryResult<Subscription>;
}

impl Handler<CreateSubscription> for Executor {
    type Result = <CreateSubscription as Message>::Result;

    fn handle(&mut self, msg: CreateSubscription, ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::subscriptions::dsl::*;

        let uuid = Uuid::new_v4();
        let subscription = NewSubscription {
            id: &uuid.to_string(),
            feed_url: &msg.feed_url,
            title: &msg.title,
        };

        diesel::insert_into(subscriptions)
            .values(&subscription)
            .execute(&self.conn)?;

        self.handle(GetSubscription(uuid), ctx)
    }
}


pub struct GetSubscription(pub Uuid);

impl Message for GetSubscription {
    type Result = diesel::QueryResult<Subscription>;
}

impl Handler<GetSubscription> for Executor {
    type Result = <GetSubscription as Message>::Result;

    fn handle(&mut self, query: GetSubscription, _: &mut Self::Context) -> Self::Result {
        use crate::schema::subscriptions::dsl::*;

        subscriptions
            .find(query.0.to_string())
            .get_result(&self.conn)
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


pub struct UpdateSubscription(pub Subscription);

impl Message for UpdateSubscription {
    type Result = diesel::QueryResult<Subscription>;
}

impl Handler<UpdateSubscription> for Executor {
    type Result = <UpdateSubscription as Message>::Result;

    fn handle(&mut self, query: UpdateSubscription, _: &mut Self::Context) -> Self::Result {
        let subscription = query.0;

        diesel::update(&subscription)
            .set(&subscription)
            .execute(&self.conn)
            .map(|_| subscription)
    }
}
