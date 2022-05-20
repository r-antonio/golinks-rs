#[macro_use]
extern crate rocket;
mod cache;
mod db;
mod golink;
mod log;

use cache::Cache;
use db::Links;
use golink::{GoLink, Id};
use std::result::Result;

use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::State;
use rocket_db_pools::Connection;

#[get("/")]
fn index(cache: &State<Cache>) -> Json<Vec<GoLink>> {
    Json(cache.get_values())
}

#[get("/<id>")]
async fn go(id: Id, cache: &State<Cache>, conn: Connection<Links>) -> Option<Redirect> {
    match cache.get_link(&id) {
        Some(value) => {
            info!("Hit cache for {}", *id);
            Some(value)
        },
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

#[launch]
fn rocket() -> _ {
    log::setup_logger().expect("Unable to initialize logger!");

    let state: Cache = Cache::new();
    rocket::build()
        .attach(db::stage())
        .attach(cache::stage())
        .mount("/", routes![index, go, post_link])
        .manage(state)
}
