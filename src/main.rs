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

    use std::convert::TryFrom;
    let consts = consts::Consts::try_from("consts.toml".as_ref())?;
    ph!("active langs\n{:?}", consts.get_active_langs());

    // get all projects that may be worked with
    let _dirs: Vec<dir_info::DirInfo> = (&consts).into();

    // ph!("{:#?}", _dirs.get(0).unwrap());
    // panic!(fh!("nois: {:?}", &true));

    // run web-server
    web::run().expect(&fh!());

    ph!("exiting pdfgen..");
    Ok(())
}
