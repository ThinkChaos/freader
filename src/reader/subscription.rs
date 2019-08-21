use actix::prelude::*;
use actix_web::{web, HttpResponse};
use actix_web::dev::HttpServiceFactory;
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

fn list(data: web::Data<crate::Data>) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    data.db
        .clone()
        .get_subscriptions()
        .from_err()
        .and_then(|subscriptions| {
            HttpResponse::Ok().json(
                ListResponse {
                    subscriptions: &subscriptions
                        .iter()
                        .map(|s| ListResponseItem {
                            id: &s.id,
                            title: &s.title,
                        }).collect::<Vec<_>>(),
                }
            )
        })
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

fn quickadd(query: web::Query<QuickAddQuery>, data: web::Data<crate::Data>) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    data.db
        .clone()
        .create_subscription(query.url.clone())
        .from_err()
        .and_then(|subscription| {
            HttpResponse::Ok().json(
                QuickAddResponse {
                    query: &subscription.feed_url,
                    stream_id: &subscription.id,
                    num_results: 1,
                }
            )
        })
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

fn edit(data: web::Data<crate::Data>, mut form: web::Form<EditData>) -> Box<dyn Future<Item = HttpResponse, Error = actix_web::Error>> {
    Box::new(data.db
        .clone()
        .transform_subscription(form.id.clone(), move |subscription| {
            if form.title.is_some() {
                subscription.title = form.title.take().unwrap();
            }
        })
        .from_err()
        .map(|_| HttpResponse::Ok().body("OK"))
    )
}
