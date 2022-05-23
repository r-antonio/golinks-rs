use crate::cache::Cache;
use crate::db::{self, Links};
use crate::golink::{GoLink, Id};
use std::result::Result;

use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::State;
use rocket_db_pools::Connection;

use tracing::{error, warn};

#[tracing::instrument(name = "GET /", skip(cache))]
#[get("/")]
fn index(cache: &State<Cache>) -> Json<Vec<GoLink>> {
    Json(cache.get_values())
}

async fn get_golink_by_id(
    id: &Id,
    cache: &State<Cache>,
    conn: Connection<Links>,
) -> Option<GoLink> {
    if let Some(value) = cache.get_link(&id) {
        debug!("Hit cache for {}", id.0);
        return Some(value);
    }

    debug!("Miss cache for {}", id.0);

    if let Some(value) = db::get_link_url(&id, conn).await {
        if let Err(message) = cache.put_link(value.clone()) {
            warn!("Could't save link into the cache: {}", message)
        };
        return Some(value);
    }

    debug!("Couldn't find any {}. Trying fuzzy search...", id.0);
    cache.get_fuzzy(&id)
}

#[tracing::instrument(name = "GET /<id>", skip(cache, conn))]
#[get("/<id>")]
async fn get_link(id: Id, cache: &State<Cache>, conn: Connection<Links>) -> Option<Redirect> {
    get_golink_by_id(&id, cache, conn)
        .await
        .map(|link| Redirect::to(link.url().clone()))
}

#[tracing::instrument(name = "POST /", skip(conn))]
#[post("/", format = "json", data = "<link>")]
async fn post_link(conn: Connection<Links>, link: GoLink) -> Result<(), &'static str> {
    db::post_link(link, conn).await.or_else(|message| {
        error!("{}", message);
        Err(message)
    })
}

#[tracing::instrument(name = "DELETE /<id>", skip(cache, conn))]
#[delete("/<id>")]
async fn delete_link(
    id: Id,
    cache: &State<Cache>,
    conn: Connection<Links>,
) -> Result<(), &'static str> {
    cache.delete_link(&id)?;
    db::delete_link(&id, conn).await?;
    Ok(())
}

pub fn all() -> Vec<rocket::Route> {
    routes![index, get_link, post_link, delete_link]
}
