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

use actix_web as web;
use failure::Error;
use std::env;

type VS = Vec<String>;
type OVS = Option<Vec<String>>;

fn main() -> Result<(), Error> {
    env_logger::init().unwrap();

    use std::convert::TryFrom;
    let consts = consts::Consts::try_from("consts.toml".as_ref())?;
    ph!("active langs\n{:?}", consts.get_active_langs());

    // get all projects that may be worked with
    let _dirs: Vec<dir_info::DirInfo> = (&consts).into();

    // rayon config
    {
        // env::set_var("RAYON_RS_NUM_CPUS", format!("{}", consts.num_cpu));
    }

    // panic!(fh!("nois: {:?}", &true));

    // run web-server
    {
        ph!("starting web-server");
        web::server::new(|| web::App::new().resource("/", |r| r.f(handler::index)))
            .bind("127.0.0.1:8088")
            .unwrap()
            .run();
    }

    ph!("finished..");
    Ok(())
}

mod handler {

    use actix_web as web;

    pub fn index(_req: &web::HttpRequest) -> &'static str {
        "Hello world!"
    }
}
