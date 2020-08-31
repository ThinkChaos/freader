use actix_web::dev::HttpServiceFactory;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::prelude::*;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/stream")
        .route("/items/ids", web::get().to(item_ids))
}

#[derive(Debug, Deserialize)]
struct ItemIdsQuery {
    #[serde(rename = "s")]
    stream: Stream,
    #[serde(rename = "xt")]
    exclude: Option<Stream>,
    #[serde(rename = "n")]
    count: usize,
}

#[derive(Debug, Serialize)]
struct ItemIdsResponse<'a> {
    #[serde(rename = "itemRefs")]
    item_refs: &'a Vec<ItemIdsResponseItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    continuation: Option<db::Id>,
}

#[derive(Debug, Serialize)]
struct ItemIdsResponseItem {
    #[serde(serialize_with = "item_id::short")]
    id: db::Id,
    #[serde(rename = "timestampUsec")]
    timestamp_usec: String,
}

async fn item_ids(
    data: web::Data<AppData>,
    query: web::Query<ItemIdsQuery>,
) -> actix_web::Result<HttpResponse> {
    use Stream::*;

    let mut db = data.db.clone();

    if matches!(&query.exclude, Some(excluded) if excluded == &query.stream) {
        return Ok(HttpResponse::BadRequest().body("Same value for s and xt"));
    }

    let is_read = match (&query.stream, &query.exclude) {
        (Read, _) => Some(true),
        (_, Some(Read)) => Some(false),
        _ => None,
    };

    let is_starred = match (&query.stream, &query.exclude) {
        (Starred, _) => Some(true),
        (_, Some(Starred)) => Some(false),
        _ => None,
    };

    let items = db.find_items(is_read, is_starred, query.count).await?;
    let item_refs = items
        .into_iter()
        .map(|item| ItemIdsResponseItem {
            id: item.id,
            timestamp_usec: (item.published.timestamp() * 1_000_000
                + item.published.timestamp_subsec_micros() as i64)
                .to_string(),
        })
        .collect();

    Ok(HttpResponse::Ok().json(ItemIdsResponse {
        item_refs: &item_refs,
        continuation: None,
    }))
}

/// A Stream represents a set of items.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[derive(Hash, Eq, PartialEq)]
#[serde(into = "String", try_from = "String")]
enum Stream {
    /// All unread items.
    Unread,
    /// All read items.
    Read,
    /// All starred items.
    Starred,
    /// All items with a tag/in a folder.
    UserLabel(String),
    /// All items from a feed.
    Feed(db::Id),
}

impl std::convert::Into<String> for Stream {
    fn into(self) -> String {
        use Stream::*;

        match self {
            Unread => "user/-/state/com.google/reading-list".to_owned(),
            Read => "user/-/state/com.google/read".to_owned(),
            Starred => "user/-/state/com.google/starred".to_owned(),
            UserLabel(l) => format!("user/-/label/{}", l),
            Feed(id) => format!("feed/{}", id.inner()),
        }
    }
}

impl std::convert::TryFrom<String> for Stream {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        use Stream::*;

        Ok(match value.as_str() {
            "user/-/state/com.google/reading-list" => Unread,
            "user/-/state/com.google/read" => Read,
            "user/-/state/com.google/starred" => Starred,
            s if s.starts_with("user/-/label/") => UserLabel(value[13..].to_owned()),
            s if s.starts_with("feed/") => UserLabel(value[5..].to_owned()),
            _ => return Err(format!("Invalid stream ID: {}", value)),
        })
    }
}

/// Specialized serialization for item IDs.
mod item_id {
    use serde::Serializer;

    use crate::prelude::*;

    /// Serialize an ID in its short (decimal) form.
    pub fn short<S: Serializer>(id: &db::Id, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&id.inner().to_string())
    }

    /// Serialize an ID in its long form.
    pub fn long<S: Serializer>(id: &db::Id, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!(
            "tag:google.com,2005:reader/item/{:016x}",
            id.inner()
        ))
    }
}
