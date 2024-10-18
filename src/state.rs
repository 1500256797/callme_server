use axum::extract::FromRef;
use sqlx::sqlite::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: SqlitePool,
}

impl FromRef<AppState> for SqlitePool {
    fn from_ref(app_state: &AppState) -> SqlitePool {
        app_state.db_pool.clone()
    }
}
