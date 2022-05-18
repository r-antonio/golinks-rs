use crate::golink::{GoLink, Id};
use rocket::fairing::{self, AdHoc};
use rocket::{Build, Rocket};
use rocket_db_pools::{sqlx, Connection, Database};
use sqlx::pool::Pool;

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

pub async fn post_link(
    link: GoLink,
    mut conn: Connection<Links>,
) -> std::result::Result<(), &'static str> {
    sqlx::query!(
        "INSERT INTO golinks (name, url) VALUES (?, ?)",
        link.name.0,
        link.url
    )
    .execute(&mut *conn)
    .await
    .unwrap();
    Ok(())
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Links::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("db/migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLite database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("SQLite Stage", |rocket| async {
        rocket
            .attach(Links::init())
            .attach(AdHoc::try_on_ignite("SQLite Migrations", run_migrations))
    })
}
