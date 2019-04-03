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

extern crate actix_web;
extern crate env_logger;
extern crate image;
extern crate rayon;
extern crate regex;
extern crate semver;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate toml;

#[macro_use]
mod macros;

mod consts;
mod dir_info;
mod info;
mod temp;
mod web;

use failure::Error;

type VS = Vec<String>;
type OVS = Option<Vec<String>>;

fn main() -> Result<(), Error> {
    env_logger::init().unwrap();

    use std::path::PathBuf;
    let consts_path = PathBuf::from("consts.toml");
    let static_path = PathBuf::from("web_server/static/");
    web::run_with(consts_path, static_path).expect(&fh!());

    ph!("exiting pdfgen..");
    Ok(())
}
