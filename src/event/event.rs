use std::sync::Arc;
use uuid::Uuid;
use chrono::DateTime;
use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use r2d2_sqlite::rusqlite::{Error, params};

#[derive(Serialize, Deserialize)]
struct Event {
    id: Uuid,
    location: String,
    time: DateTime<Utc>,
    description: String
}

pub fn create_event(state: Arc<join::AppState>, event: &Event) -> Result<(), Error> {
    state.conn_pool.get().unwrap().execute(
        "INSERT INTO event (id, location, time, description)
        VALUES ({?1}, {?2}, {?3}, {?4})",
        params![event.id.to_string(), event.location, event.time, event.description],
    )?;
    Ok(())
}
