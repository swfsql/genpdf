use actix_web as aweb;
use failure::Error;
use std::cell::Cell;

mod handler;

pub struct AppState {
    counter: Cell<usize>,
}

impl AppState {
    fn new() -> Self {
        AppState {
            counter: Cell::new(0),
        }
    }
}

pub fn run() -> Result<(), Error> {
    ph!("starting web-server");
    let _server = aweb::server::new(|| {
        vec![
            aweb::App::with_state(AppState::new())
                .prefix("/app1")
                .resource("/", |r| r.f(handler::index_state))
                .resource("/", |r| r.f(handler::index_state))
                .boxed(),
            aweb::App::new()
                .handler(
                    "/static/",
                    aweb::fs::StaticFiles::new("web_server/static/")
                        .expect(&fh!())
                        .show_files_listing(),
                )
                .boxed(),
        ]
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run();
    ph!("web-server closed");
    Ok(())
}
