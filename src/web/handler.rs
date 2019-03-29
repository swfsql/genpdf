use super::AppState;
use actix_web as aweb;

pub fn index_state(req: &aweb::HttpRequest<AppState>) -> String {
    let count = req.state().counter.get() + 1; // <- get count
    req.state().counter.set(count); // <- store new count in state

    format!("Request number: {}", count) // <- response with count
}

pub fn index(_req: &aweb::HttpRequest) -> &'static str {
    "Hello world!"
}
