use axum::{
    routing::{get, post},
    Router,
    extract::{State, Form},
    response::Html,
};
use std::sync::Arc;
use r2d2_sqlite::SqliteConnectionManager;
use minijinja::{Environment, context};
use serde::Deserialize;
use chrono::DateTime;
use chrono::prelude::*;

mod event;
mod attendee;

#[derive(Deserialize)]
#[derive(Debug)]
struct NewEventForm {
    location: String,
    #[serde(rename="timeUtc")]
    time_utc: DateTime<Utc>,
    desc: String,
    #[serde(rename="firstName")]
    first_name: String,
    #[serde(rename="lastName")]
    last_name: String
}

#[tokio::main]
async fn main() {
    let manager = SqliteConnectionManager::file("./join.db");
    let pool = r2d2::Pool::new(manager).unwrap();
    let shared_state = Arc::new(join::AppState {
        conn_pool: pool
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/new_event", get(new_event))
        .route("/new_event", post(new_event_post))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn make_env() -> Environment<'static> {
    let mut env = Environment::new();
    env.set_loader(minijinja::path_loader("templates"));
    env
}

async fn root(State(_state): State<Arc<join::AppState>>) -> Html<String> {
    let env = make_env();
    let home_template = env.get_template("home.html").unwrap();
    Html(home_template.render(()).unwrap())
}

async fn new_event(State(_state): State<Arc<join::AppState>>) -> Html<String> {
    let env = make_env();
    let new_event_template = env.get_template("new_event.html").unwrap();
    Html(new_event_template.render(()).unwrap())
}

async fn new_event_post(State(_state): State<Arc<join::AppState>>, Form(new_event_form): Form<NewEventForm>) -> Html<String> {
    dbg!(new_event_form);
    let env = make_env();
    let new_event_template = env.get_template("new_event.html").unwrap();
    Html(new_event_template.render(()).unwrap())
}
