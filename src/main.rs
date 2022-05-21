#[macro_use]
extern crate rocket;

mod cache;
mod db;
mod golink;
mod routes;
mod tracing;

use cache::Cache;
use rocket::fairing::AdHoc;

#[launch]
fn rocket() -> _ {
    tracing::init_tracer();

    rocket::build()
        .attach(db::stage())
        .attach(cache::stage())
        .mount("/", routes::all())
        .manage(Cache::new())
        .attach(AdHoc::on_shutdown(
            "Shutdown Tracer",
            tracing::shutdown_tracer,
        ))
}
