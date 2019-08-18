use actix::prelude::*;
use actix_web::{web, HttpResponse};
use actix_web::dev::HttpServiceFactory;
use serde::{Deserialize, Serialize};

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
    id: String,
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
    let uuid = match form.id.parse() {
        Ok(u) => u,
        Err(_) => return Box::new(futures::future::ok(HttpResponse::BadRequest().body("Invalid subscription id"))),
    };

    Box::new(data.db
        .clone()
        .transform_subscription(uuid, move |subscription| {
            if form.title.is_some() {
                subscription.title = form.title.take().unwrap();
            }
        })
        .from_err()
        .map(|_| HttpResponse::Ok().body("OK"))
    )
}
