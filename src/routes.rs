use crate::cache::Cache;
use crate::db::{self, Links};
use crate::golink::{GoLink, Id};
use std::result::Result;

use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::State;
use rocket_db_pools::Connection;

use tracing::{error, info, warn};

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
        .map(|link| Redirect::to(link.url))
}

#[tracing::instrument(name = "POST /", skip(conn))]
#[post("/", data = "<link>")]
async fn post_link(conn: Connection<Links>, link: Json<GoLink>) -> Result<(), &'static str> {
    db::post_link(link.0, conn).await.or_else(|message| {
        error!("{}", message);
        Err(message)
    })
}

pub fn all() -> Vec<rocket::Route> {
    routes![index, get_link, post_link]
}
