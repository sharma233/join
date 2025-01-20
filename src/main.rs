mod event;
mod attendee;

use axum::handler::HandlerWithoutStateExt;
use attendee::attendee::{create_attendee, get_attendees_by_event_id, Attendee};
use axum::{
    http::{StatusCode, Uri, header},
    routing::{get, post},
    Router,
    extract::{State, Form, Path, Host},
    response::{Html, Redirect, IntoResponse, Response},
    BoxError,
};
use axum_server::tls_rustls::RustlsConfig;
use std::sync::Arc;
use std::fs;
use r2d2_sqlite::SqliteConnectionManager;
use minijinja::{Environment, context};
use serde::Deserialize;
use chrono::DateTime;
use chrono::prelude::*;
use uuid::Uuid;
use event::event::{get_event_by_id, Event};
use event::event::create_event;
use std::net::SocketAddr;
use std::error::Error;
use ics::{properties::{DtStart, Location, Summary, TzID}, Event as ICSEvent, ICalendar};

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

#[derive(Clone, Copy)]
struct Ports {
    http: u16,
    https: u16
}

pub enum AppError {
    Err(Box<dyn Error>),
    MinijinjaErr(minijinja::Error)
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, render_error()).into_response()
    }
}

impl From<Box<dyn Error>> for AppError {
    fn from(err: Box<dyn Error>) -> Self {
        Self::Err(err)
    }
}

impl From<minijinja::Error> for AppError {
    fn from(err: minijinja::Error) -> Self {
        Self::MinijinjaErr(err)
    }
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
        .route("/ics/:event_id", get(generate_ics))
        .with_state(shared_state);

    let ports = Ports {
        http: 3000,
        https: 7878
    };

    tokio::spawn(redirect_http_to_https(ports));
    let config = RustlsConfig::from_pem_file("cert.pem", "key.pem").await.unwrap();
    let addr = SocketAddr::from(([0, 0, 0, 0], ports.https));
    axum_server::bind_rustls(addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn make_env() -> Environment<'static> {
    let mut env = Environment::new();
    env.set_loader(minijinja::path_loader("templates"));
    env
}

async fn root(State(_state): State<Arc<join::AppState>>) -> Result<Html<String>, AppError> {
    let env = make_env();
    let home_template = env.get_template("home.html")?;
    match home_template.render(()) {
        Ok(template) => Ok(Html(template)),
        Err(e) => Err(e.into())
    }
}

async fn new_event(State(_state): State<Arc<join::AppState>>) -> Result<Html<String>, AppError> {
    let env = make_env();
    let new_event_template = env.get_template("new_event.html")?;
    match new_event_template.render(()) {
        Ok(template) => Ok(Html(template)),
        Err(e) => Err(e.into())
    }
}

async fn new_attendee(State(_state): State<Arc<join::AppState>>) -> Result<Html<String>, AppError> {
    let env = make_env();
    let new_attendee_template = env.get_template("new_attendee.html")?;
    match new_attendee_template .render(()) {
        Ok(template) => Ok(Html(template)),
        Err(e) => Err(e.into())
    }
}

fn render_error() -> Html<String> {
    let env = make_env();
    let error_template = env.get_template("error.html").unwrap();
    Html(error_template.render(()).unwrap())
}

async fn view_event(State(state): State<Arc<join::AppState>>, Path(event_id): Path<Uuid>) -> Result<Html<String>, AppError> {
    let env = make_env();
    let new_event_template = env.get_template("event_page.html").unwrap();
    
    let event = get_event_by_id(&state, event_id)?;

    let attendees = get_attendees_by_event_id(&state, event_id)?;
    let page = context! {
        event => event,
        attendees => attendees
    };

    Ok(Html(new_event_template.render(context!(page)).unwrap()))
}

async fn new_event_post(State(state): State<Arc<join::AppState>>, Form(new_event_form): Form<NewEventForm>) -> Result<Redirect, AppError> {
    let uuid = Uuid::new_v4();
    let event = Event { 
        id: uuid,
        location: new_event_form.location,
        time: new_event_form.time_utc,
        description: new_event_form.desc
    };

    create_event(&state, &event)?;

    let a_uuid = Uuid::new_v4();
    let attendee = Attendee {
        id: a_uuid,
        event_id: uuid,
        first_name: new_event_form.first_name,
        last_name: new_event_form.last_name,
    };

    create_attendee(&state, &attendee)?;

    let redir_url = format!("/event/{}", uuid);
    Ok(Redirect::to(&redir_url))
}


async fn new_attendee_post(State(state): State<Arc<join::AppState>>, Path(event_id): Path<Uuid>, Form(new_attendee_form): Form<NewAttendeeForm>) -> Redirect {
    let a_uuid = Uuid::new_v4();
    let attendee = Attendee {
        id: a_uuid,
        event_id,
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

//pretty much lifted directly from: https://github.com/tokio-rs/axum/blob/main/examples/tls-rustls/src/main.rs
//maybe revisit at some point
async fn redirect_http_to_https(ports: Ports) {
    fn make_https(host: String, uri: Uri, ports: Ports) -> Result<Uri, BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        let https_host = host.replace(&ports.http.to_string(), &ports.https.to_string());
        parts.authority = Some(https_host.parse()?);

        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(host, uri, ports) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(_error) => {
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], ports.http));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, redirect.into_make_service())
        .await
        .unwrap();
}

async fn generate_ics(State(state): State<Arc<join::AppState>>, Path(event_id): Path<Uuid>) -> Result<impl IntoResponse, StatusCode> {
    let event = match get_event_by_id(&state, event_id){
        Ok(res) => res,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR)
    };

    let ics_time = event.time.format("%Y%m%dT%H%M%SZ").to_string();
    let mut ics_event = ICSEvent::new(event.id.to_string(), &ics_time);
    ics_event.push(Summary::new(event.description));
    ics_event.push(Location::new(event.location));
    ics_event.push(DtStart::new(&ics_time));
    ics_event.push(TzID::new("/Etc/UTC"));

    let mut calendar = ICalendar::new("2.0", "ics-rs");
    calendar.add_event(ics_event);
    match calendar.save_file("cal.ics"){
        Ok(_) => {},
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR)
    };

    let data = match fs::read("cal.ics"){
        Ok(res) => {res},
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR)
    };
    let stream = axum::body::Body::from(data);
    match fs::remove_file("cal.ics") {
        Ok(_) => {},
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR)
    };

    let headers = [
        (header::CONTENT_TYPE, "text/calendar"),
        (
            header::CONTENT_DISPOSITION,
            &format!("attachment; filename=\"{:?}\"", "calendar"),
        ),
    ];

    Ok((headers, stream).into_response())
}
