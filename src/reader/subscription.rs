use actix_web::dev::HttpServiceFactory;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::db::models::Category;
use crate::prelude::*;

pub fn service() -> impl HttpServiceFactory {
    web::scope("/subscription")
        .route("/edit", web::post().to(edit))
        .route("/list", web::get().to(list))
        .route("/quickadd", web::post().to(quickadd))
}


#[derive(Debug, Serialize)]
struct ListResponse<'a> {
    subscriptions: &'a Vec<ListResponseItem<'a>>,
}

#[derive(Debug, Serialize)]
struct ListResponseItem<'a> {
    id: SubscriptionId,
    title: &'a str,
    #[serde(rename = "htmlUrl", skip_serializing_if = "Option::is_none")]
    site_url: &'a Option<String>,
    categories: Vec<ListResponseCategoryItem<'a>>,
}

#[derive(Debug, Serialize)]
struct ListResponseCategoryItem<'a> {
    id: LabelId,
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
                id: LabelId(category.name.clone()),
                label: &category.name,
            });

            ListResponseItem {
                id: SubscriptionId(subscription.id),
                title: &subscription.title,
                site_url: &subscription.site_url,
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
    stream_id: SubscriptionId,
    query: &'a str,
    #[serde(rename = "numResults")]
    num_results: u8,
}

#[derive(Debug, Serialize)]
struct QuickAddErrorResponse<'a> {
    query: &'a str,
    #[serde(rename = "numResults")]
    num_results: u8,
    error: &'a str,
}

async fn quickadd(
    data: web::Data<AppData>,
    query: web::Query<QuickAddQuery>,
) -> actix_web::Result<HttpResponse> {
    let subscription = data.feed_manager.subscribe(&query.url).await.map_err(|e| {
        HttpResponse::Ok().json(QuickAddErrorResponse {
            query: &query.url,
            num_results: 0,
            error: e,
        })
    })?;

    Ok(HttpResponse::Ok().json(QuickAddResponse {
        query: &subscription.feed_url,
        stream_id: SubscriptionId(subscription.id),
        num_results: 1,
    }))
}


#[derive(Debug, Deserialize)]
struct EditData {
    #[serde(rename = "s")]
    id: SubscriptionId,
    #[serde(rename = "ac")]
    action: String,
    #[serde(rename = "t")]
    title: Option<String>,
    #[serde(rename = "a")]
    add_category: Option<LabelId>,
    #[serde(rename = "r")]
    remove_category: Option<LabelId>,
}

async fn edit(
    data: web::Data<AppData>,
    mut form: web::Form<EditData>,
) -> actix_web::Result<HttpResponse> {
    let mut db = data.db.clone();

    match form.action.as_str() {
        "edit" => {
            if let Some(title) = form.title.take() {
                db.transform_subscription(form.id.0, move |subscription| {
                    subscription.title = title;
                })
                .await?;
            }

            if form.add_category != form.remove_category {
                if let Some(category) = form.add_category.take() {
                    db.subscription_add_category(form.id.0, category.0).await?;
                }

                if let Some(category) = form.remove_category.take() {
                    db.subscription_remove_category(form.id.0, category.0)
                        .await?;
                }
            }
        }
        "unsubscribe" => {
            db.remove_subscription(form.id.0).await?;
        }
        _ => return Ok(HttpResponse::BadRequest().body("Bad value for ac")),
    }

    Ok(HttpResponse::Ok().body("OK"))
}

pub const SUBSCRIPTION_ID_PREFIX: &str = "feed/";

/// A subscription is a feed.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[derive(Hash, Eq, PartialEq)]
#[serde(into = "String", try_from = "&str")]
pub struct SubscriptionId(pub db::Id);

impl std::convert::Into<String> for SubscriptionId {
    fn into(self) -> String {
        format!("{}{}", SUBSCRIPTION_ID_PREFIX, self.0.inner())
    }
}

impl<'a> std::convert::TryFrom<&'a str> for SubscriptionId {
    type Error = String;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if value.starts_with(SUBSCRIPTION_ID_PREFIX) {
            Ok(Self(
                value[SUBSCRIPTION_ID_PREFIX.len()..]
                    .parse::<db::Id>()
                    .map_err(|e| e.to_string())?,
            ))
        } else {
            Err(format!("Invalid feed ID: {}", value))
        }
    }
}


pub const LABEL_ID_PREFIX: &str = "user/-/label/";

/// A label identifies a folder or a tag.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[derive(Hash, Eq, PartialEq)]
#[serde(into = "String", try_from = "&str")]
pub struct LabelId(pub String);

impl std::convert::Into<String> for LabelId {
    fn into(self) -> String {
        format!("{}{}", LABEL_ID_PREFIX, self.0)
    }
}

impl<'a> std::convert::TryFrom<&'a str> for LabelId {
    type Error = String;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if value.starts_with(LABEL_ID_PREFIX) {
            Ok(Self(value[LABEL_ID_PREFIX.len()..].to_owned()))
        } else {
            Err(format!("Invalid tag/folder ID: {}", value))
        }
    }
}
