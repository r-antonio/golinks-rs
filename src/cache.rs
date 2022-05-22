use crate::db::{self, Links};
use crate::golink::{GoLink, Id};
use fuse_rust::Fuse;
use rocket::fairing::{self, AdHoc};
use rocket::{Build, Rocket};
use rocket_db_pools::Database;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{error, info, warn};

pub struct Cache {
    data: Arc<RwLock<HashMap<Id, GoLink>>>,
    fuse: Fuse,
}

impl std::fmt::Debug for Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            data: Arc::new(RwLock::new(HashMap::new())),
            fuse: Fuse::default(),
        }
    }

    pub fn set_values(&self, links: Vec<GoLink>) -> Result<(), &'static str> {
        let mut new_map: HashMap<Id, GoLink> = HashMap::new();
        for link in links {
            new_map.insert(link.name.clone(), link);
        }
        let mut ref_map = self.data.write().unwrap();
        *ref_map = new_map;
        Ok(())
    }

    #[tracing::instrument(name = "cache::get_link", skip(self))]
    pub fn get_link(&self, id: &Id) -> Option<GoLink> {
        self.data.read().unwrap().get(id).map(Clone::clone)
    }

    #[tracing::instrument(name = "cache::put_link", skip(self))]
    pub fn put_link(&self, link: GoLink) -> Result<(), &'static str> {
        match self.data.write() {
            Ok(mut lock) => {
                lock.insert(link.name.clone(), link);
                Ok(())
            }
            Err(_) => Err("Couldn't add link to the cache"),
        }
    }

    #[tracing::instrument(name = "cache::delete_link", skip(self))]
    pub fn delete_link(&self, id: &Id) -> Result<(), &'static str> {
        if let Ok(mut lock) = self.data.write() {
            match lock.remove(id) {
                Some(_) => return Ok(()),
                _ => {}
            }
        }
        Err("Could't delete entry from cache")
    }

    #[tracing::instrument(name = "cache::get_values", skip(self))]
    pub fn get_values(&self) -> Vec<GoLink> {
        self.data.read().unwrap().values().cloned().collect()
    }

    #[tracing::instrument(name = "cache::get_suggested_links", skip(self))]
    pub fn get_fuzzy(&self, id: &Id) -> Option<GoLink> {
        let items = self.get_values();
        let results = self.fuse.search_text_in_fuse_list(&id.0, &items);
        match results.len() {
            1 => {
                let value = items.get(results.get(0).unwrap().index).unwrap().clone();
                info!("Matched {} for input: {}", *value.name, **id);
                Some(value)
            }
            num_matched_values => {
                info!("Matched {} values for input: {}", num_matched_values, **id);
                None
            }
        }
    }
}

async fn load_initial_data(rocket: Rocket<Build>) -> fairing::Result {
    let db = Links::fetch(&rocket).unwrap();
    let pool = &**db;
    match db::get_all_links(pool).await {
        Ok(links) => match rocket.state::<Cache>().unwrap().set_values(links) {
            Ok(_) => {
                info!("Cache loaded successfully");
                Ok(rocket)
            }
            Err(message) => {
                warn!("Couldn't load cache data {}", message);
                Err(rocket)
            }
        },
        Err(message) => {
            error!("{}", message);
            Err(rocket)
        }
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Cache Stage", |rocket| async {
        rocket.attach(AdHoc::try_on_ignite("Cache Warmup", load_initial_data))
    })
}
