use r2d2_sqlite::SqliteConnectionManager;


pub struct AppState {
    pub conn_pool: r2d2::Pool<SqliteConnectionManager>
}
