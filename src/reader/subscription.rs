use actix_web::dev::HttpServiceFactory;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::models::Category;
use crate::prelude::*;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/subscription")
        .route("/list", web::get().to(list))
        .route("/quickadd", web::post().to(quickadd))
        .route("/edit", web::post().to(edit))
}


#[derive(Debug, Serialize)]
struct ListResponse<'a> {
    subscriptions: &'a Vec<ListResponseItem<'a>>,
}

#[derive(Debug, Serialize)]
struct ListResponseItem<'a> {
    id: &'a db::Id,
    title: &'a str,
    categories: Vec<ListResponseCategoryItem<'a>>,
}

#[derive(Debug, Serialize)]
struct ListResponseCategoryItem<'a> {
    id: &'a db::Id,
    label: &'a str,
}

async fn list(data: web::Data<AppData>) -> actix_web::Result<HttpResponse> {
    let mut db = data.db.clone();

    let subscriptions = db.get_subscriptions().await?;

    let mut categories: Vec<Vec<Category>> = Vec::with_capacity(subscriptions.len());
    for subscription in &subscriptions {
        categories.push(db.get_subscription_categories(subscription.id).await?);
    }

    let subscriptions = subscriptions
        .iter()
        .zip(&categories)
        .map(|(subscription, categories)| {
            let categories = categories.iter().map(|category| ListResponseCategoryItem {
                id: &category.id,
                label: &category.name,
            });

            ListResponseItem {
                id: &subscription.id,
                title: &subscription.title,
                categories: categories.collect(),
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(ListResponse {
        subscriptions: &subscriptions,
    }))
}


#[derive(Debug, Deserialize)]
struct QuickAddQuery {
    #[serde(rename = "quickadd")]
    url: String,
}

#[derive(Debug, Serialize)]
struct QuickAddResponse<'a> {
    #[serde(rename = "streamId")]
    stream_id: &'a db::Id,
    query: &'a str,
    #[serde(rename = "numResults")]
    num_results: u8,
}

async fn quickadd(
    data: web::Data<AppData>,
    query: web::Query<QuickAddQuery>,
) -> actix_web::Result<HttpResponse> {
    let subscription = data
        .db
        .clone()
        .create_subscription(query.url.clone())
        .await?;

    Ok(HttpResponse::Ok().json(QuickAddResponse {
        query: &subscription.feed_url,
        stream_id: &subscription.id,
        num_results: 1,
    }))
}


#[derive(Debug, Deserialize)]
struct EditData {
    #[serde(rename = "s")]
    id: db::Id,
    // #[serde(rename="ac")]
    // operation: Option<String>, // "edit"
    #[serde(rename = "t")]
    title: Option<String>,
    #[serde(rename = "a")]
    add_category: Option<String>,
    #[serde(rename = "r")]
    remove_category: Option<String>,
}

async fn edit(
    data: web::Data<AppData>,
    mut form: web::Form<EditData>,
) -> actix_web::Result<HttpResponse> {
    let mut db = data.db.clone();

    if form.title.is_some() {
        let title = form.title.take().unwrap();

        db.transform_subscription(form.id, move |subscription| {
            subscription.title = title;
        })
        .await?;
    }

    if form.add_category != form.remove_category {
        if form.add_category.is_some() {
            let category = form.add_category.take().unwrap();

            db.subscription_add_category(form.id, category).await?;
        }

        if form.remove_category.is_some() {
            let category = form.remove_category.take().unwrap();

            db.subscription_remove_category(form.id, category).await?;
        }
    }

    Ok(HttpResponse::Ok().body("OK"))
}
