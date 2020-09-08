use actix::prelude::*;
use diesel::prelude::*;
use std::rc::Rc;

use crate::db::{self, models::*, schema};

pub struct Executor {
    conn: Rc<SqliteConnection>,
}

impl Executor {
    pub fn connect(connspec: &str) -> ConnectionResult<Self> {
        Ok(Executor {
            conn: Rc::new(SqliteConnection::establish(connspec)?),
        })
    }
}

impl Actor for Executor {
    type Context = SyncContext<Self>;
}


/// Message containing a query builder to be executed with `.load`.
pub struct FindAll<F, Q, T>(F, std::marker::PhantomData<T>)
where
    F: FnOnce() -> Q,
    Q: diesel::query_dsl::LoadQuery<SqliteConnection, T>;

impl<F, Q, T> FindAll<F, Q, T>
where
    F: FnOnce() -> Q,
    Q: diesel::query_dsl::LoadQuery<SqliteConnection, T>,
{
    pub fn new(query_builder: F) -> Self {
        Self(query_builder, Default::default())
    }
}

impl<F, Q, T> Message for FindAll<F, Q, T>
where
    F: FnOnce() -> Q,
    Q: diesel::query_dsl::LoadQuery<SqliteConnection, T>,
    T: 'static,
{
    type Result = diesel::QueryResult<Vec<T>>;
}

impl<F, Q, T> Handler<FindAll<F, Q, T>> for Executor
where
    F: FnOnce() -> Q,
    Q: diesel::query_dsl::LoadQuery<SqliteConnection, T>,
    T: 'static,
{
    type Result = <FindAll<F, Q, T> as Message>::Result;

    fn handle(&mut self, msg: FindAll<F, Q, T>, _: &mut Self::Context) -> Self::Result {
        let query = msg.0();

        query.load(self.conn.as_ref())
    }
}


pub struct CreateSubscription(pub NewSubscription);

impl Message for CreateSubscription {
    type Result = diesel::QueryResult<Subscription>;
}

impl Handler<CreateSubscription> for Executor {
    type Result = <CreateSubscription as Message>::Result;

    fn handle(&mut self, msg: CreateSubscription, _: &mut Self::Context) -> Self::Result {
        self.conn.transaction(|| {
            use schema::subscriptions::dsl::*;

            diesel::insert_into(subscriptions)
                .values(&msg.0)
                .execute(self.conn.as_ref())?;

            subscriptions.order(id.desc()).first(self.conn.as_ref())
        })
    }
}


pub struct RemoveSubscription(pub db::Id);

impl Message for RemoveSubscription {
    type Result = diesel::QueryResult<()>;
}

impl Handler<RemoveSubscription> for Executor {
    type Result = <RemoveSubscription as Message>::Result;

    fn handle(&mut self, msg: RemoveSubscription, ctx: &mut Self::Context) -> Self::Result {
        use schema::items::dsl::*;
        use schema::subscriptions::dsl::*;

        self.conn.clone().transaction(|| {
            if let Some(subscription) = self.handle(GetSubscription(msg.0), ctx).optional()? {
                // Remove subscription's categories
                let categories = self.handle(GetSubscriptionCategories(subscription.id), ctx)?;
                for category in categories {
                    self.handle(
                        SubscriptionRemoveCategory {
                            subscription_id: subscription.id,
                            category_name: category.name,
                        },
                        ctx,
                    )?;
                }

                // Remove subscription's items
                diesel::delete(items.filter(subscription_id.eq(subscription.id)))
                    .execute(self.conn.as_ref())?;

                // Remove subscription
                diesel::delete(subscriptions.find(subscription.id)).execute(self.conn.as_ref())?;
            }

            Ok(())
        })
    }
}


pub struct GetSubscription(pub db::Id);

impl Message for GetSubscription {
    type Result = diesel::QueryResult<Subscription>;
}

impl Handler<GetSubscription> for Executor {
    type Result = <GetSubscription as Message>::Result;

    fn handle(&mut self, msg: GetSubscription, _: &mut Self::Context) -> Self::Result {
        use schema::subscriptions::dsl::*;

        subscriptions.find(msg.0).get_result(self.conn.as_ref())
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
            .execute(self.conn.as_ref())
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

        self.conn.clone().transaction(|| {
            let mut subscription = self.handle(GetSubscription(id), ctx)?;

            transform(&mut subscription);

            self.handle(UpdateSubscription(subscription), ctx)
        })
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

    fn handle(&mut self, msg: CreateCategory, _: &mut Self::Context) -> Self::Result {
        let category = NewCategory { name: &msg.name };

        self.conn.transaction(|| {
            use schema::categories::dsl::*;

            diesel::insert_into(categories)
                .values(&category)
                .execute(self.conn.as_ref())?;

            categories.order(id.desc()).first(self.conn.as_ref())
        })
    }
}


pub struct GetCategory(pub db::Id);

impl Message for GetCategory {
    type Result = diesel::QueryResult<Category>;
}

impl Handler<GetCategory> for Executor {
    type Result = <GetCategory as Message>::Result;

    fn handle(&mut self, msg: GetCategory, _: &mut Self::Context) -> Self::Result {
        use schema::categories::dsl::*;

        categories.find(msg.0).get_result(self.conn.as_ref())
    }
}


pub struct GetCategoryByName(pub String);

impl Message for GetCategoryByName {
    type Result = diesel::QueryResult<Option<Category>>;
}

impl Handler<GetCategoryByName> for Executor {
    type Result = <GetCategoryByName as Message>::Result;

    fn handle(&mut self, msg: GetCategoryByName, _: &mut Self::Context) -> Self::Result {
        use schema::categories::dsl::*;

        let maybe_category = categories
            .filter(name.eq(&msg.0))
            .limit(1)
            .load(self.conn.as_ref())?
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
        self.conn.clone().transaction(|| {
            self.handle(GetCategoryByName(msg.name.clone()), ctx)?
                .map(Ok)
                .unwrap_or_else(|| self.handle(CreateCategory { name: msg.name }, ctx))
        })
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
        use schema::subscription_categories::dsl::subscription_categories;

        let SubscriptionAddCategory {
            subscription_id,
            category_name,
        } = msg;

        self.conn.clone().transaction(|| {
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
                .execute(self.conn.as_ref())?;

            Ok(category)
        })
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
        use schema::subscription_categories::dsl::{category_id, subscription_categories};

        let SubscriptionRemoveCategory {
            subscription_id,
            category_name,
        } = msg;

        self.conn.clone().transaction(|| {
            let category = self.handle(GetCategoryByName(category_name), ctx)?;

            if let Some(category) = category {
                diesel::delete(subscription_categories.find((subscription_id, &category.id)))
                    .execute(self.conn.as_ref())?;

                let n: i64 = subscription_categories
                    .filter(category_id.eq(category.id))
                    .count()
                    .get_result(self.conn.as_ref())?;

                if n == 0 {
                    use schema::categories::dsl::categories;

                    diesel::delete(categories.find(category.id)).execute(self.conn.as_ref())?;
                }
            }

            Ok(())
        })
    }
}


pub struct GetSubscriptionCategories(pub db::Id);

impl Message for GetSubscriptionCategories {
    type Result = diesel::QueryResult<Vec<Category>>;
}

impl Handler<GetSubscriptionCategories> for Executor {
    type Result = <GetSubscriptionCategories as Message>::Result;

    fn handle(&mut self, msg: GetSubscriptionCategories, _: &mut Self::Context) -> Self::Result {
        use schema::categories::dsl::*;
        use schema::subscription_categories::dsl::*;

        subscription_categories
            .filter(subscription_id.eq(&msg.0))
            .inner_join(categories)
            .select(categories::all_columns())
            .load(self.conn.as_ref())
    }
}


pub struct CreateItem(pub NewItem);

impl Message for CreateItem {
    type Result = diesel::QueryResult<Item>;
}

impl Handler<CreateItem> for Executor {
    type Result = <CreateItem as Message>::Result;

    fn handle(&mut self, msg: CreateItem, _: &mut Self::Context) -> Self::Result {
        self.conn.transaction(|| {
            use schema::items::dsl::*;

            diesel::insert_into(items)
                .values(&msg.0)
                .execute(self.conn.as_ref())?;

            items.order(id.desc()).first(self.conn.as_ref())
        })
    }
}
