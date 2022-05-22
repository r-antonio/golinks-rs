use fuse_rust::{FuseProperty, Fuseable};
use rocket::data::{Data, FromData, Outcome};
use rocket::http::Status;
use rocket::request::{FromParam, Request};
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use url::Url;

use std::clone::Clone;
use std::cmp::{Eq, PartialEq};
use std::ops::Deref;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct GoLink {
    pub url: String,
    pub name: Id,
}

#[rocket::async_trait]
impl<'r> FromData<'r> for GoLink {
    type Error = &'r str;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> Outcome<'r, Self> {
        match Json::<GoLink>::from_data(req, data).await {
            Outcome::Success(json_data) => match Url::parse(&json_data.url) {
                Ok(url) => {
                    debug!("Parsed URL : {:?}", url);
                    Outcome::Success(GoLink {
                        name: json_data.name.clone(),
                        url: json_data.url.clone(),
                    })
                }
                Err(e) => {
                    info!("Malformed URL: {}", e);
                    Outcome::Failure((Status::BadRequest, "Field `url` is not a valid URL"))
                }
            },
            Outcome::Failure(err) => {
                debug!("Failed to parse json : {:?}", err);
                Outcome::Failure((Status::BadRequest, "Couldn't convert Json to GoLink"))
            }
            // Json doesn't implement Outcome::Forward
            Outcome::Forward(_) => Outcome::Failure((Status::BadRequest, "")),
        }
    }
}

impl Fuseable for GoLink {
    fn properties(&self) -> Vec<FuseProperty> {
        vec![FuseProperty {
            value: String::from("id"),
            weight: 1.0,
        }]
    }

    fn lookup(&self, key: &str) -> Option<&str> {
        match key {
            "id" => Some(&self.name),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
#[serde(crate = "rocket::serde")]
pub struct Id(pub String);

impl Deref for Id {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}

impl<'a> FromParam<'a> for Id {
    type Error = &'a str;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        param
            .chars()
            .all(|c| c.is_ascii_alphanumeric())
            .then(|| Id(param.into()))
            .ok_or(param)
    }
}
