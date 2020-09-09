use actix_web::dev::HttpServiceFactory;
use actix_web::web;

use actix_web::HttpResponse;
use serde::Deserialize;

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

#[derive(Debug, Deserialize)]
struct EditTagForm {
    #[serde(rename = "i")]
    item_id: ItemId,
    #[serde(rename = "a")]
    add_tag: Option<StreamId>,
    #[serde(rename = "r")]
    remove_tag: Option<StreamId>,
}

async fn edit_tag(
    data: web::Data<AppData>,
    form: web::Form<EditTagForm>,
) -> actix_web::Result<HttpResponse> {
    let mut db = data.db.clone();

    let mut item = db.get_item(form.item_id.0).await?;

    match form.add_tag {
        Some(StreamId::Read) => item.is_read = true,
        Some(StreamId::Starred) => item.is_starred = true,
        _ => (),
    }

    match form.remove_tag {
        Some(StreamId::Read) => item.is_read = false,
        Some(StreamId::Starred) => item.is_starred = false,
        _ => (),
    }

    db.update_item(item).await?;

    Ok(HttpResponse::Ok().body("OK"))
}
