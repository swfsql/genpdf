#![warn(unused_extern_crates)]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate tera;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate strum_macros;
#[macro_use]
extern crate custom_derive;
#[macro_use]
extern crate newtype_derive;

extern crate image;
extern crate regex;
extern crate semver;
extern crate serde;
extern crate strum;

extern crate isolang;
extern crate language_tags;
extern crate whatlang;

#[macro_use]
mod macros;

pub mod consts;
pub mod dir_info;
mod info;
pub mod temp;
// mod web;

type VS = Vec<String>;
type OVS = Option<Vec<String>>;
