use actix_web::{web, HttpResponse};
use actix_web::dev::HttpServiceFactory;
use actix_web_async_compat::async_compat;
use futures_03::{compat::Future01CompatExt, FutureExt, TryFutureExt};
use serde::{Deserialize, Serialize};

use crate::db;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/subscription")
        .route("/list", web::get().to_async(list))
        .route("/quickadd", web::post().to_async(quickadd))
        .route("/edit", web::post().to_async(edit))
}


#[derive(Debug, Serialize)]
struct ListResponse<'a> {
    subscriptions: &'a [ListResponseItem<'a>],
}

#[derive(Debug, Serialize)]
struct ListResponseItem<'a> {
  id: &'a db::Id,
  title: &'a str,
}

#[async_compat]
async fn list(data: web::Data<crate::Data>) -> actix_web::Result<HttpResponse> {
    let subscriptions = data.db
        .clone()
        .get_subscriptions()
        .compat()
        .await?;

    Ok(HttpResponse::Ok().json(
        ListResponse {
            subscriptions: &subscriptions
                .iter()
                .map(|s| ListResponseItem {
                    id: &s.id,
                    title: &s.title,
                }).collect::<Vec<_>>(),
        }
    ))
}


#[derive(Debug, Deserialize)]
struct QuickAddQuery {
    #[serde(rename="quickadd")]
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

#[async_compat]
async fn quickadd(query: web::Query<QuickAddQuery>, data: web::Data<crate::Data>) -> actix_web::Result<HttpResponse> {
    let subscription = data.db
        .clone() // FIXME
        .create_subscription(query.url.clone())
        .compat()
        .await?;

    Ok(HttpResponse::Ok().json(
        QuickAddResponse {
            query: &subscription.feed_url,
            stream_id: &subscription.id,
            num_results: 1,
        }
    ))
}


#[derive(Debug, Deserialize)]
struct EditData {
    #[serde(rename="s")]
    id: db::Id,
    // #[serde(rename="ac")]
    // operation: Option<String>, // "edit"
    #[serde(rename="t")]
    title: Option<String>,
    #[serde(rename="a")]
    add_category: Option<String>,
    #[serde(rename="r")]
    remove_category: Option<String>,
}

#[async_compat]
async fn edit(data: web::Data<crate::Data>, mut form: web::Form<EditData>) -> actix_web::Result<HttpResponse> {
    if form.title.is_some() {
        data.db
            .clone()
            .transform_subscription(form.id.clone(), move |subscription| {
                subscription.title = form.title.take().unwrap();
            })
            .compat()
            .await?;
    }

    Ok(HttpResponse::Ok().body("OK"))
}
