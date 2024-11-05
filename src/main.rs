mod event;
mod attendee;

use attendee::attendee::{create_attendee, get_attendees_by_event_id, Attendee};
use axum::{
    routing::{get, post},
    Router,
    extract::{State, Form, Path},
    response::{Html, Redirect}
};
use std::sync::Arc;
use r2d2_sqlite::SqliteConnectionManager;
use minijinja::{Environment, context};
use serde::Deserialize;
use chrono::DateTime;
use chrono::prelude::*;
use uuid::Uuid;
use event::event::{get_event_by_id, Event};
use event::event::create_event;

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

#[derive(Deserialize)]
#[derive(Debug)]
struct NewAttendeeForm {
    #[serde(rename="firstName")]
    first_name: String,
    #[serde(rename="lastName")]
    last_name: String
}

#[tokio::main]
async fn main() {
    let manager = SqliteConnectionManager::file("./join.db");
    //should panic
    let pool = r2d2::Pool::new(manager).unwrap();
    let shared_state = Arc::new(join::AppState {
        conn_pool: pool
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/new_event", get(new_event))
        .route("/new_event", post(new_event_post))
        .route("/event/:event_id", get(view_event))
        .route("/new_attendee/:event_id", get(new_attendee))
        .route("/new_attendee/:event_id", post(new_attendee_post))
        .with_state(shared_state);

    //should panic
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    //should panic
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

async fn new_attendee(State(_state): State<Arc<join::AppState>>) -> Html<String> {
    let env = make_env();
    let new_attendee_template = env.get_template("new_attendee.html").unwrap();
    Html(new_attendee_template.render(()).unwrap())
}

fn render_error() -> Html<String> {
    let env = make_env();
    let error_template = env.get_template("error.html").unwrap();
    Html(error_template.render(()).unwrap())
}

async fn view_event(State(state): State<Arc<join::AppState>>, Path(event_id): Path<Uuid>) -> Html<String> {
    let env = make_env();
    let new_event_template = env.get_template("event_page.html").unwrap();
    
    let event = match get_event_by_id(&state, event_id) {
        Ok(res) => res,
        Err(e) => {
            dbg!(e); 
            return render_error()
        }
    };

    let attendees = match get_attendees_by_event_id(&state, event_id) {
        Err(e) => {
            dbg!(e);
            return render_error()
        },
        Ok(res) => res
    };
    let page = context! {
        event => event,
        attendees => attendees
    };

    Html(new_event_template.render(context!(page)).unwrap())
}

async fn new_event_post(State(state): State<Arc<join::AppState>>, Form(new_event_form): Form<NewEventForm>) -> Redirect {
    let uuid = Uuid::new_v4();
    let event = Event { 
        id: uuid,
        location: new_event_form.location,
        time: new_event_form.time_utc,
        description: new_event_form.desc
    };

    match create_event(&state, &event) {
        Err(e) => {
            dbg!(e);
            return Redirect::to("/error/")
        },
        Ok(_) => {}
    };

    let a_uuid = Uuid::new_v4();
    let attendee = Attendee {
        id: a_uuid,
        event_id: uuid,
        first_name: new_event_form.first_name,
        last_name: new_event_form.last_name,
    };

    match create_attendee(&state, &attendee) {
        Err(e) => {
            dbg!(e);
            return Redirect::to("/error/")
        },
        Ok(_) => {}
    }

    let redir_url = format!("/event/{}", uuid);
    Redirect::to(&redir_url)
}


async fn new_attendee_post(State(state): State<Arc<join::AppState>>, Path(event_id): Path<Uuid>, Form(new_attendee_form): Form<NewAttendeeForm>) -> Redirect {
    let a_uuid = Uuid::new_v4();
    let attendee = Attendee {
        id: a_uuid,
        event_id: event_id,
        first_name: new_attendee_form.first_name,
        last_name: new_attendee_form.last_name,
    };

    match create_attendee(&state, &attendee) {
        Err(e) => {
            dbg!(e);
            return Redirect::to("/error/")
        },
        Ok(_) => {}
    }

    let redir_url = format!("/event/{}", event_id);
    Redirect::to(&redir_url)
}
