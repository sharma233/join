use std::sync::Arc;
use r2d2_sqlite::SqliteConnectionManager;

mod event;

fn main() {
    let manager = SqliteConnectionManager::file("./backlog.db");
    let pool = r2d2::Pool::new(manager).unwrap();

    let shared_state = Arc::new(join::AppState {
        conn_pool: pool
    });
}
