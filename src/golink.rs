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
    url: String,
    name: Id,
}

impl GoLink {
    pub fn new(name: Id, url: String) -> Option<GoLink> {
        match Url::parse(&url) {
            Ok(_) => Some(GoLink { name, url }),
            Err(_) => None,
        }
    }

    pub fn url(&self) -> &String {
        &self.url
    }

    pub fn name(&self) -> &Id {
        &self.name
    }
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

#[cfg(test)]
mod tests {
    use crate::golink::*;

    #[test]
    fn alphanumeric_param() {
        let param: &str = "validCharacters345";
        assert!(Id::from_param(param).is_ok());
    }

    #[test]
    fn non_alphanumeric_param() {
        let param: &str = "invalid/charact*r%";
        assert!(Id::from_param(param).is_err());
    }

    fn test_golink(url: String) -> Option<GoLink> {
        GoLink::new(Id("test".to_string()), url)
    }

    #[test]
    fn create_invalid_golink() {
        let url = String::from("some-random-string");
        assert!(test_golink(url).is_none());
    }

    fn test_valid_url(url: &str) {
        assert!(test_golink(String::from(url)).is_some());
    }

    #[test]
    fn create_https_golink() {
        test_valid_url("https://domain.org");
    }

    #[test]
    fn create_ftp_with_port_golink() {
        test_valid_url("ftp://domain.org:400");
    }

    #[test]
    fn create_query_params_golink() {
        test_valid_url("http://some.org/dir?q=1&r=3");
    }

    #[test]
    fn create_file_golink() {
        test_valid_url("file:///home/user/Documents/book.pdf");
    }

    #[test]
    fn create_golinkception() {
        test_valid_url("https://go/test")
    }
}
