use actix::prelude::*;
use actix_web::{web, HttpResponse};
use actix_web::dev::HttpServiceFactory;
use serde::{Deserialize, Serialize};

pub fn service() -> impl HttpServiceFactory {
    web::scope("/subscription")
        .route("/list", web::get().to_async(list))
        .route("/quickadd", web::post().to_async(quickadd))
}


#[derive(Debug, Serialize)]
struct ListResponse<'a> {
    subscriptions: &'a [ListResponseItem<'a>],
}

#[derive(Debug, Serialize)]
struct ListResponseItem<'a> {
  id: &'a str,
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
    stream_id: &'a str,
    query: &'a str,
    #[serde(rename = "numResults")]
    num_results: u8,
}

fn quickadd(info: web::Query<QuickAddQuery>, data: web::Data<crate::Data>) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    data.db
        .clone()
        .create_subscription(info.url.clone())
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
