use crate::db::{self, Links};
use crate::golink::{GoLink, Id};
use fuse_rust::Fuse;
use rocket::fairing::{self, AdHoc};
use rocket::{Build, Rocket};
use rocket_db_pools::Database;
use std::collections::HashMap;
use std::sync::RwLock;

pub struct Cache {
    data: RwLock<HashMap<Id, GoLink>>,
    fuse: Fuse,
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            data: RwLock::new(HashMap::new()),
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

    pub fn get_link(&self, id: &Id) -> Option<GoLink> {
        self.data.read().unwrap().get(id).map(Clone::clone)
    }

    pub fn put_link(&self, link: GoLink) -> Result<(), &'static str> {
        match self.data.write() {
            Ok(mut lock) => {
                lock.insert(link.name.clone(), link);
                Ok(())
            }
            Err(_) => Err("Couldn't add link to the cache"),
        }
    }

    pub fn get_values(&self) -> Vec<GoLink> {
        self.data.read().unwrap().values().cloned().collect()
    }

    pub fn get_fuzzy(&self, id: &Id) -> Option<GoLink> {
        let items = self.get_values();
        let results = self.fuse.search_text_in_fuse_list(&id.0, &items);
        match results.len() {
            1 => results
                .first()
                .map(move |r| items.get(r.index).unwrap().clone()),
            _ => None,
        }
    }
}

async fn load_initial_data(rocket: Rocket<Build>) -> fairing::Result {
    let db = Links::fetch(&rocket).unwrap();
    let pool = &**db;
    match db::get_all_links(pool).await {
        Ok(links) => {
            if let Ok(_) = rocket.state::<Cache>().unwrap().set_values(links) {
                return Ok(rocket);
            }
            Err(rocket)
        }
        Err(_) => Err(rocket),
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Cache Stage", |rocket| async {
        rocket.attach(AdHoc::try_on_ignite("Cache Warmup", load_initial_data))
    })
}
