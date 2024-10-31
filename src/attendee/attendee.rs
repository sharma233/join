use std::sync::Arc;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use r2d2_sqlite::rusqlite::{Error, params};

#[derive(Serialize, Deserialize)]
pub struct Attendee {
    pub id: Uuid,
    pub event_id: Uuid,
    pub first_name: String,
    pub last_name: String
}

pub fn create_attendee(state: Arc<join::AppState>, attendee: &Attendee) -> Result<(), Error> {
    state.conn_pool.get().unwrap().execute(
        "INSERT INTO attendee (id, event_id, first_name, last_name)
        VALUES ({?1}, {?2}, {?3}, {?4})",
        params![attendee.id.to_string(), attendee.event_id.to_string(), attendee.first_name, attendee.last_name],
    )?;
    Ok(())
}
