use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;
use chrono::DateTime;
use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use r2d2_sqlite::rusqlite::params;
use r2d2_sqlite::rusqlite::named_params;
use std::error::Error;

#[derive(Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub location: String,
    pub time: DateTime<Utc>,
    pub description: String
}

pub fn create_event(state: &Arc<join::AppState>, event: &Event) -> Result<(), Box<dyn Error>> {
    state.conn_pool.get()?.execute(
        "INSERT INTO event (id, location, time, description)
        VALUES (?1, ?2, ?3, ?4)",
        params![event.id.to_string(), event.location, event.time, event.description],
    )?;
    Ok(())
}

pub fn get_event_by_id(state: &Arc<join::AppState>, event_id: Uuid) -> Result<Event, Box<dyn Error>> {
    let conn = state.conn_pool.get()?;
    let mut stmt = conn.prepare("SELECT id, location, time, description FROM event WHERE id = :id")?;
    let mut rows = stmt.query(named_params! {":id": event_id.to_string().as_str()})?;

    let row = rows.next()?;
    if let Some(row) = row {
        let ids: String = row.get(0)?;
        let idu: Uuid = Uuid::from_str(ids.as_str())?;
        Ok(
            Event {
                id: idu,
                location: row.get(1)?,
                time: row.get(2)?,
                description: row.get(3)?,
            }
        )
    } else {
        Err(Box::from("event not found"))
    }
}
