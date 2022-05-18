use fuse_rust::{FuseProperty, Fuseable};
use rocket::request::FromParam;
use rocket::serde::{Deserialize, Serialize};

use std::clone::Clone;
use std::cmp::{Eq, PartialEq};
use std::ops::Deref;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(crate = "rocket::serde")]
pub struct GoLink {
    pub url: String,
    pub name: Id,
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
