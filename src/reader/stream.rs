use actix_web::dev::HttpServiceFactory;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use super::subscription::{LabelId, SubscriptionId, LABEL_ID_PREFIX, SUBSCRIPTION_ID_PREFIX};
use crate::prelude::*;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/stream")
        .route("/items/contents", web::post().to(item_contents))
        .route("/items/ids", web::get().to(item_ids))
}

#[derive(Debug, Deserialize)]
struct ItemIdsQuery {
    #[serde(rename = "s")]
    stream: StreamId,
    #[serde(rename = "xt")]
    exclude: Option<StreamId>,
    #[serde(rename = "n")]
    count: usize,
}

#[derive(Debug, Serialize)]
struct ItemIdsResponse<'a> {
    #[serde(rename = "itemRefs")]
    item_refs: &'a Vec<ItemIdsResponseItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    continuation: Option<String>,
}

#[derive(Debug, Serialize)]
struct ItemIdsResponseItem {
    #[serde(serialize_with = "item_id::short")]
    id: ItemId,
    #[serde(rename = "timestampUsec")]
    timestamp_usec: String,
}

async fn item_ids(
    data: web::Data<AppData>,
    query: web::Query<ItemIdsQuery>,
) -> actix_web::Result<HttpResponse> {
    use StreamId::{Read, Starred};

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
            id: ItemId(item.id),
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

// serde_urlencoded doesn't support repeated items because it is non-standard.
// We must manually parse the key/value pairs.
type ItemContentsForm = Vec<(String, String)>;

#[derive(Debug, Serialize)]
struct ItemContentsResponse<'a> {
    items: &'a Vec<ItemContentsResponseItem<'a>>,
}

#[derive(Debug, Serialize)]
struct ItemContentsResponseItem<'a> {
    #[serde(serialize_with = "item_id::long")]
    id: ItemId,
    title: &'a str,
    author: &'a str,
    published: i64,
    updated: i64,
    #[serde(rename = "timestampUsec")]
    timestamp_usec: String,
    #[serde(rename = "alternate")]
    link: Vec<ItemContentsResponseItemLink<'a>>,
    origin: ItemContentsResponseItemOrigin<'a>,
    summary: ItemContentsResponseItemSummary<'a>,
}

#[derive(Debug, Serialize)]
struct ItemContentsResponseItemLink<'a> {
    href: &'a str,
}

#[derive(Debug, Serialize)]
struct ItemContentsResponseItemOrigin<'a> {
    #[serde(rename = "streamId")]
    id: SubscriptionId,
    title: &'a str,
    #[serde(rename = "htmlUrl", skip_serializing_if = "Option::is_none")]
    site_url: &'a Option<String>,
}

#[derive(Debug, Serialize)]
struct ItemContentsResponseItemSummary<'a> {
    content: &'a str,
}

async fn item_contents(
    data: web::Data<AppData>,
    form: web::Form<ItemContentsForm>,
) -> actix_web::Result<HttpResponse> {
    let mut db = data.db.clone();

    // Manually extract the IDs
    let ids = {
        let mut ids = Vec::with_capacity(form.len());
        for (k, v) in form.iter() {
            if k == "i" {
                match ItemId::try_from(v.to_owned()) {
                    Ok(id) => ids.push(id.0),
                    Err(_) => return Ok(HttpResponse::BadRequest().body("Invalid item ID")),
                }
            }
        }
        ids
    };

    let pairs = db.get_items_and_subscriptions(ids).await?;

    let items = pairs
        .iter()
        .map(|(item, subscription)| ItemContentsResponseItem {
            id: ItemId(item.id),
            title: &item.title,
            author: &item.author,
            published: item.published.timestamp(),
            updated: item.updated.timestamp(),
            timestamp_usec: (item.published.timestamp() * 1_000_000
                + item.published.timestamp_subsec_micros() as i64)
                .to_string(),
            link: vec![ItemContentsResponseItemLink {
                href: &item.url,
            }],
            origin: ItemContentsResponseItemOrigin {
                id: SubscriptionId(item.subscription_id),
                title: &subscription.title,
                site_url: &subscription.site_url,
            },
            summary: ItemContentsResponseItemSummary {
                content: &item.content,
            },
        })
        .collect();

    Ok(HttpResponse::Ok().json(ItemContentsResponse {
        items: &items,
    }))
}


/// A Stream represents a set of items.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[derive(Hash, Eq, PartialEq)]
#[serde(into = "String", try_from = "String")]
pub enum StreamId {
    /// All unread items.
    Unread,
    /// All read items.
    Read,
    /// All starred items.
    Starred,
    /// All items with a tag/in a folder.
    UserLabel(LabelId),
    /// All items from a subscription.
    Subscription(SubscriptionId),
}

impl std::convert::Into<String> for StreamId {
    fn into(self) -> String {
        use StreamId::*;

        match self {
            Unread => "user/-/state/com.google/reading-list".to_owned(),
            Read => "user/-/state/com.google/read".to_owned(),
            Starred => "user/-/state/com.google/starred".to_owned(),
            UserLabel(id) => id.into(),
            Subscription(id) => id.into(),
        }
    }
}

impl TryFrom<String> for StreamId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        use StreamId::*;

        Ok(match value.as_str() {
            "user/-/state/com.google/reading-list" => Unread,
            "user/-/state/com.google/read" => Read,
            "user/-/state/com.google/starred" => Starred,
            s if s.starts_with(LABEL_ID_PREFIX) => UserLabel(LabelId::try_from(value)?),
            s if s.starts_with(SUBSCRIPTION_ID_PREFIX) => {
                Subscription(SubscriptionId::try_from(value.to_owned())?)
            }
            _ => return Err(format!("Invalid stream ID: {}", value)),
        })
    }
}


pub const LONG_ITEM_ID_PREFIX: &'static str = "tag:google.com,2005:reader/item/";

/// An item is an entry in a feed.
///
/// `ItemId` doesn't implement `Serialize` to ensure either the short or
/// the long form is specified on each struct field using: `serialize_with`.
#[derive(Debug, Clone, Deserialize)]
#[derive(Hash, Eq, PartialEq)]
#[serde(try_from = "String")]
pub struct ItemId(pub db::Id);

impl TryFrom<String> for ItemId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let (base, value) = match value {
            // Long form with prefix
            _ if value.starts_with(LONG_ITEM_ID_PREFIX) => {
                (16, &value[LONG_ITEM_ID_PREFIX.len()..])
            }
            // Long form without prefix: hex, 0 padded to 16 chars
            // Note: a base 10 number with 16 digits is too big to fit an i32,
            // so this must be hex.
            _ if value.len() == 16 => (16, value.as_str()),
            // Short form: base 10 number
            _ => (10, value.as_str()),
        };

        i32::from_str_radix(value, base)
            .map(|id| ItemId(db::Id::from_raw(id)))
            .map_err(|e| e.to_string())
    }
}

/// Specialized serialization for item IDs.
mod item_id {
    use serde::Serializer;

    use super::ItemId;

    /// Serialize an ID in its short (decimal) form.
    pub fn short<S: Serializer>(id: &ItemId, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&id.0.inner().to_string())
    }

    /// Serialize an ID in its long form.
    pub fn long<S: Serializer>(id: &ItemId, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!(
            "tag:google.com,2005:reader/item/{:016x}",
            id.0.inner()
        ))
    }
}
