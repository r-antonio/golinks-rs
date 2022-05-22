use crate::golink::{GoLink, Id};
use rocket::fairing::{self, AdHoc};
use rocket::{Build, Rocket};
use rocket_db_pools::{sqlx, Connection, Database};
use sqlx::pool::Pool;
use tracing::error;

#[derive(Database)]
#[database("golinks")]
pub struct Links(sqlx::SqlitePool);

pub async fn get_all_links(pool: &Pool<sqlx::Sqlite>) -> Result<Vec<GoLink>, &'static str> {
    let results = sqlx::query!("SELECT name, url FROM golinks")
        .fetch_all(pool)
        .await
        .ok();
    if let Some(records) = results {
        let links: Vec<GoLink> = records
            .iter()
            .map(|r| GoLink {
                name: Id(r.name.clone()),
                url: r.url.clone(),
            })
            .collect();
        Ok(links)
    } else {
        Err("Couldn't load links from database")
    }
}

#[tracing::instrument(name = "db::get_link", skip(conn))]
pub async fn get_link_url(id: &Id, mut conn: Connection<Links>) -> Option<GoLink> {
    sqlx::query!("SELECT name, url FROM golinks WHERE name = ?", id.0)
        .fetch_one(&mut *conn)
        .await
        .map(|r| GoLink {
            name: Id(r.name),
            url: r.url,
        })
        .ok()
}

#[tracing::instrument(name = "db::post_link", skip(conn))]
pub async fn post_link(
    link: GoLink,
    mut conn: Connection<Links>,
) -> Result<(), &'static str> {
    match sqlx::query!(
        "INSERT INTO golinks (name, url) VALUES (?, ?)",
        link.name.0,
        link.url
    )
    .execute(&mut *conn)
    .await {
        Err(message) => {
            error!("Failed to insert link : {}", message);
            Err("Couldn't insert link in database")
        },
        _ => Ok(())
    }
}

#[tracing::instrument(name = "db::delete_link", skip(conn))]
pub async fn delete_link(id: &Id, mut conn: Connection<Links>) -> Result<(), &'static str> {
    match sqlx::query!("DELETE FROM golinks WHERE name = ?", id.0)
    .execute(&mut *conn)
    .await {
        Err(message) => {
            error!("Failed to delete link : {}", message);
            Err("Couldn't delete link from database")
        },
        _ => Ok(())
    }
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Links::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("db/migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to run migrations: {}", e);
                Err(rocket)
            }
        },
        None => {
            error!("Failed to fetch database");
            Err(rocket)
        }
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("SQLite Stage", |rocket| async {
        rocket
            .attach(Links::init())
            .attach(AdHoc::try_on_ignite("SQLite Migrations", run_migrations))
    })
}
