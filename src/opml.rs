use chrono::TimeZone;
use futures::future::LocalBoxFuture;

use crate::db::models::NewSubscription;
use crate::prelude::*;

/// Import feeds and categories from `file`.
pub async fn import(file: &str, db: &mut db::Helper) -> std::io::Result<()> {
    let xml = std::fs::read_to_string(file)?;
    let opml =
        opml::OPML::new(&xml).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    import_outlines(opml.body.outlines, db, None).await?;

    Ok(())
}

fn import_outlines<'a>(
    outlines: Vec<opml::Outline>,
    db: &'a mut db::Helper,
    category: Option<String>,
) -> LocalBoxFuture<'a, std::io::Result<()>> {
    Box::pin(async move {
        for outline in outlines {
            let title = outline.title.unwrap_or(outline.text);

            let feed_url = match outline.xml_url {
                Some(url) => url,
                None => {
                    if let Some(parent) = category {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!(
                                "Nested categories are invalid: {} contains {}",
                                parent, title
                            ),
                        ));
                    }

                    // No feed: this outline is a category
                    import_outlines(outline.outlines, db, Some(title)).await?;
                    continue;
                }
            };

            let result = db
                .create_subscription(NewSubscription {
                    feed_url: feed_url.clone(),
                    title: title.clone(),
                    site_url: outline.html_url,
                    refreshed_at: chrono::Utc.timestamp(0, 0).naive_utc(),
                })
                .await;

            let subscription = match result {
                Ok(subscription) => subscription,
                Err(db::Error::DatabaseError(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ))) => {
                    log::warn!("Skipping {} ({}): already in database", title, feed_url);
                    continue;
                }
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    ))
                }
            };

            log::info!("Added {} ({})", title, feed_url);

            if let Some(category) = &category {
                db.subscription_add_category(subscription.id, category.clone())
                    .await
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
            }
        }

        Ok(())
    })
}
