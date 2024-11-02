use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use r2d2_sqlite::rusqlite::params;
use r2d2_sqlite::rusqlite::named_params;
use std::error::Error;

#[derive(Serialize, Deserialize)]
pub struct Attendee {
    pub id: Uuid,
    pub event_id: Uuid,
    pub first_name: String,
    pub last_name: String
}

pub fn create_attendee(state: &Arc<join::AppState>, attendee: &Attendee) -> Result<(), Box<dyn Error>> {
    state.conn_pool.get().unwrap().execute(
        "INSERT INTO attendee (id, event_id, first_name, last_name)
        VALUES (?1, ?2, ?3, ?4)",
        params![attendee.id.to_string(), attendee.event_id.to_string(), attendee.first_name, attendee.last_name],
    )?;
    Ok(())
}

pub fn get_attendees_by_event_id(state: &Arc<join::AppState>, event_id: Uuid) -> Result<Vec<Attendee>, Box<dyn Error>> {
    let conn = state.conn_pool.get()?;
    let mut stmt = conn.prepare("SELECT id, event_id, first_name, last_name FROM attendee WHERE event_id = :id")?;
    let mut rows = stmt.query(named_params!{":id": event_id.to_string().as_str()})?;

    let mut attendees: Vec<Attendee> = Vec::new();
    while let Some(row) = rows.next()? {
        let ids: String = row.get(0)?;
        let idu: Uuid = Uuid::from_str(ids.as_str())?;

        let e_ids: String = row.get(1)?;
        let e_idu: Uuid = Uuid::from_str(e_ids.as_str())?;

        let attendee = Attendee {
            id: idu,
            event_id: e_idu,
            first_name: row.get(2)?,
            last_name: row.get(3)?,
        };
        attendees.push(attendee);
    }
    Ok(attendees)
}
