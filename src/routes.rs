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

#[tracing::instrument(name = "GET /<id>", skip(cache, conn))]
#[get("/<id>")]
async fn get_link(id: Id, cache: &State<Cache>, conn: Connection<Links>) -> Option<Redirect> {
    match cache.get_link(&id) {
        Some(value) => {
            info!("Hit cache for {}", *id);
            Some(value)
        }
        None => match db::get_link_url(&id, conn).await {
            Some(value) => {
                info!("Cache miss for {}, found it in database", *id);
                if let Err(message) = cache.put_link(value.clone()) {
                    warn!("Could't save link into the cache: {}", message)
                };
                Some(value)
            }
            None => cache.get_fuzzy(&id),
        },
    }
    .map(|link| Redirect::to(link.url))
}

#[tracing::instrument(name = "POST /", skip(conn))]
#[post("/", data = "<link>")]
async fn post_link(conn: Connection<Links>, link: Json<GoLink>) -> Result<(), &'static str> {
    match db::post_link(link.0, conn).await {
        Ok(result) => Ok(result),
        Err(message) => {
            error!("{}", message);
            Err(message)
        }
    }
}

pub fn all() -> Vec<rocket::Route> {
    routes![index, get_link, post_link]
}
