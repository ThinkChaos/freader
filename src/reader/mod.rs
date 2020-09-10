use actix_web::{dev::HttpServiceFactory, web, HttpResponse};
use std::convert::TryFrom;

use crate::prelude::*;
use stream::{ItemId, StreamId};

mod stream;
mod subscription;
mod user_info;
mod utils;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/reader/api/0")
        .wrap(utils::RequireAuth)
        .service(stream::service())
        .service(subscription::service())
        .service(user_info::service())
        .route("/edit-tag", web::post().to(edit_tag))
}

async fn edit_tag(
    data: web::Data<AppData>,
    form: web::Form<Vec<(String, String)>>,
) -> actix_web::Result<HttpResponse> {
    let mut item_ids = Vec::with_capacity(form.len());

    let mut new_is_read = None;
    let mut new_is_starred = None;

    // Manually parse form because i can be repeated
    for (k, v) in form.into_inner() {
        match k.as_str() {
            "i" => {
                let id = ItemId::try_from(v.as_str()).map_err(|e| {
                    HttpResponse::BadRequest().body(format!("Invalid item id {}: {}", v, e))
                })?;
                item_ids.push(id);
            }
            "a" | "r" => {
                let id = StreamId::try_from(v.as_str()).map_err(|e| {
                    HttpResponse::BadRequest().body(format!("Invalid stream id {}: {}", v, e))
                })?;
                match id {
                    StreamId::Read => new_is_read = Some(k == "a"),
                    StreamId::Unread => new_is_read = Some(k != "a"),
                    StreamId::Starred => new_is_starred = Some(k == "a"),
                    _ => (),
                }
            }
            _ => continue,
        }
    }

    if new_is_read.is_some() || new_is_starred.is_some() {
        let mut db = data.db.clone();

        for item_id in item_ids {
            let mut item = db.get_item(item_id.0).await?;

            item.is_read = new_is_read.unwrap_or(item.is_read);
            item.is_starred = new_is_starred.unwrap_or(item.is_starred);

            db.update_item(item).await?;
        }
    }

    Ok(HttpResponse::Ok().body("OK"))
}
