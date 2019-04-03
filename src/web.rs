use crate::{consts, dir_info};
use actix_web as aweb;
use failure::Error;
use std::cell::Cell;
use std::path::{Path, PathBuf};

mod handler;

#[derive(Debug, Clone)]
pub struct AppState {
    consts: consts::Consts,
    dirs: Vec<dir_info::DirInfo>,
}

impl AppState {
    pub fn try_new(consts_path: PathBuf) -> Result<Self, Error> {
        use std::convert::TryFrom;

        // instantiate from a consts file
        let consts = consts::Consts::try_from(consts_path.as_ref())?;
        ph!("active langs\n{:?}", consts.get_active_langs());

        // get all projects that may be worked with
        let dirs: Vec<dir_info::DirInfo> = (&consts).into();

        Ok(AppState { consts, dirs })
    }
}

pub fn run_with(consts_path: PathBuf, static_path: PathBuf) -> Result<(), Error> {
    use std::sync::Arc;
    let state = Arc::new(AppState::try_new(consts_path.clone()).expect(&fh!()));
    ph!("starting web-server");
    let _server = aweb::server::new(move || {
        vec![
            aweb::App::with_state(state.clone())
                .prefix("/app1")
                .resource("/", |r| r.f(handler::temporary_index))
                .resource("/hello", |r| r.f(handler::index_state))
                .resource("/dirs", |r| r.f(handler::get_dirs))
                .resource("/gen_projs", |r| {
                    r.method(aweb::http::Method::POST).with(handler::gen_projs)
                })
                .boxed(),
            aweb::App::new()
                .handler(
                    "/static/",
                    aweb::fs::StaticFiles::new(static_path.clone())
                        .expect(&fh!())
                        .show_files_listing(),
                )
                .boxed(),
        ]
    })
    // .workers(1)
    .bind("127.0.0.1:8088")
    .expect(&fh!())
    .run();
    ph!("web-server closed");
    Ok(())
}
