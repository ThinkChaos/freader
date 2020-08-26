use actix::prelude::*;
use diesel::prelude::*;

use crate::db;
use crate::models::*;

pub struct Executor {
    conn: SqliteConnection,
}

impl Executor {
    pub fn connect(connspec: &str) -> ConnectionResult<Self> {
        Ok(Executor {
            conn: SqliteConnection::establish(connspec)?,
        })
    }
}

impl Actor for Executor {
    type Context = SyncContext<Self>;
}


pub struct CreateSubscription {
    pub feed_url: String,
    pub title: String,
    pub site_url: Option<String>,
}

impl Message for CreateSubscription {
    type Result = diesel::QueryResult<Subscription>;
}

impl Handler<CreateSubscription> for Executor {
    type Result = <CreateSubscription as Message>::Result;

    fn handle(&mut self, msg: CreateSubscription, ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::subscriptions::dsl::subscriptions;

        let id = db::Id::new();
        let subscription = NewSubscription {
            id: &id,
            feed_url: &msg.feed_url,
            title: &msg.title,
            site_url: msg.site_url.as_ref().map(String::as_str),
        };

        diesel::insert_into(subscriptions)
            .values(&subscription)
            .execute(&self.conn)?;

        self.handle(GetSubscription(id), ctx)
    }
}


pub struct GetSubscription(pub db::Id);

impl Message for GetSubscription {
    type Result = diesel::QueryResult<Subscription>;
}

impl Handler<GetSubscription> for Executor {
    type Result = <GetSubscription as Message>::Result;

    fn handle(&mut self, msg: GetSubscription, _: &mut Self::Context) -> Self::Result {
        use crate::schema::subscriptions::dsl::*;

        subscriptions.find(msg.0).get_result(&self.conn)
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

    fn handle(&mut self, msg: UpdateSubscription, _: &mut Self::Context) -> Self::Result {
        let subscription = msg.0;

        diesel::update(&subscription)
            .set(&subscription)
            .execute(&self.conn)
            .map(|_| subscription)
    }
}


pub struct TransformSubscription(pub db::Id, pub Box<dyn FnOnce(&mut Subscription) + Send>);

impl Message for TransformSubscription {
    type Result = diesel::QueryResult<Subscription>;
}

impl Handler<TransformSubscription> for Executor {
    type Result = <TransformSubscription as Message>::Result;

    fn handle(&mut self, msg: TransformSubscription, ctx: &mut Self::Context) -> Self::Result {
        let (id, transform) = (msg.0, msg.1);

        let mut subscription = self.handle(GetSubscription(id), ctx)?;

        transform(&mut subscription);

        self.handle(UpdateSubscription(subscription), ctx)
    }
}


pub struct CreateCategory {
    pub name: String,
}

impl Message for CreateCategory {
    type Result = diesel::QueryResult<Category>;
}

impl Handler<CreateCategory> for Executor {
    type Result = <CreateCategory as Message>::Result;

    fn handle(&mut self, msg: CreateCategory, ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::categories::dsl::categories;

        let id = db::Id::new();
        let category = NewCategory {
            id: &id,
            name: &msg.name,
        };

        diesel::insert_into(categories)
            .values(&category)
            .execute(&self.conn)?;

        self.handle(GetCategory(id), ctx)
    }
}


pub struct GetCategory(pub db::Id);

impl Message for GetCategory {
    type Result = diesel::QueryResult<Category>;
}

impl Handler<GetCategory> for Executor {
    type Result = <GetCategory as Message>::Result;

    fn handle(&mut self, msg: GetCategory, _: &mut Self::Context) -> Self::Result {
        use crate::schema::categories::dsl::*;

        categories.find(msg.0).get_result(&self.conn)
    }
}


pub struct GetCategoryByName(pub String);

impl Message for GetCategoryByName {
    type Result = diesel::QueryResult<Option<Category>>;
}

impl Handler<GetCategoryByName> for Executor {
    type Result = <GetCategoryByName as Message>::Result;

    fn handle(&mut self, msg: GetCategoryByName, _: &mut Self::Context) -> Self::Result {
        use crate::schema::categories::dsl::*;

        let maybe_category = categories
            .filter(name.eq(&msg.0))
            .limit(1)
            .load(&self.conn)?
            .pop();

        Ok(maybe_category)
    }
}


pub struct GetOrCreateCategory {
    pub name: String,
}

impl Message for GetOrCreateCategory {
    type Result = diesel::QueryResult<Category>;
}

impl Handler<GetOrCreateCategory> for Executor {
    type Result = <GetOrCreateCategory as Message>::Result;

    fn handle(&mut self, msg: GetOrCreateCategory, ctx: &mut Self::Context) -> Self::Result {
        self.handle(GetCategoryByName(msg.name.clone()), ctx)?
            .map(Ok)
            .unwrap_or_else(|| self.handle(CreateCategory { name: msg.name }, ctx))
    }
}


pub struct SubscriptionAddCategory {
    pub subscription_id: db::Id,
    pub category_name: String,
}

impl Message for SubscriptionAddCategory {
    type Result = diesel::QueryResult<Category>;
}

impl Handler<SubscriptionAddCategory> for Executor {
    type Result = <SubscriptionAddCategory as Message>::Result;

    fn handle(&mut self, msg: SubscriptionAddCategory, ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::subscription_categories::dsl::subscription_categories;

        let SubscriptionAddCategory {
            subscription_id,
            category_name,
        } = msg;

        let category = self.handle(
            GetOrCreateCategory {
                name: category_name,
            },
            ctx,
        )?;

        let subscription_category = NewSubscriptionCategory {
            subscription_id: &subscription_id,
            category_id: &category.id,
        };

        diesel::insert_or_ignore_into(subscription_categories)
            .values(&subscription_category)
            .execute(&self.conn)?;

        Ok(category)
    }
}


pub struct SubscriptionRemoveCategory {
    pub subscription_id: db::Id,
    pub category_name: String,
}

impl Message for SubscriptionRemoveCategory {
    type Result = diesel::QueryResult<()>;
}

impl Handler<SubscriptionRemoveCategory> for Executor {
    type Result = <SubscriptionRemoveCategory as Message>::Result;

    fn handle(&mut self, msg: SubscriptionRemoveCategory, ctx: &mut Self::Context) -> Self::Result {
        use crate::schema::subscription_categories::dsl::{category_id, subscription_categories};

        let SubscriptionRemoveCategory {
            subscription_id,
            category_name,
        } = msg;

        let category = self.handle(GetCategoryByName(category_name), ctx)?;

        if let Some(category) = category {
            diesel::delete(subscription_categories.find((subscription_id, &category.id)))
                .execute(&self.conn)?;

            let n: i64 = subscription_categories
                .filter(category_id.eq(category.id))
                .count()
                .get_result(&self.conn)?;

            if n == 0 {
                use crate::schema::categories::dsl::categories;

                diesel::delete(categories.find(category.id)).execute(&self.conn)?;
            }
        }

        Ok(())
    }
}


pub struct GetSubscriptionCategories(pub db::Id);

impl Message for GetSubscriptionCategories {
    type Result = diesel::QueryResult<Vec<Category>>;
}

impl Handler<GetSubscriptionCategories> for Executor {
    type Result = <GetSubscriptionCategories as Message>::Result;

    fn handle(&mut self, msg: GetSubscriptionCategories, _: &mut Self::Context) -> Self::Result {
        use crate::schema::categories::dsl::*;
        use crate::schema::subscription_categories::dsl::*;

        subscription_categories
            .filter(subscription_id.eq(&msg.0))
            .inner_join(categories)
            .select(categories::all_columns())
            .load(&self.conn)
    }
}
