use std::{fs::File, path::Path};

use axum::{routing::get, Router};
use sqlx::sqlite::SqlitePool;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
mod controller;
mod state;
const DB_FILE: &str = "call_phone.db";
const DB_URL: &str = "sqlite:call_phone.db";
#[tokio::main]
async fn main() {
    std::env::set_var("DATABASE_URL", DB_URL);
    if !Path::new(DB_FILE).exists() {
        File::create(DB_FILE).expect("无法创建数据库文件");
    }
    let pool = SqlitePool::connect(DB_URL).await.unwrap();
    // sql will be migrated to db only when axum app start and  sql file changed
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    println!("migration success");
    // cors
    let cors = CorsLayer::very_permissive();
    // state
    let state = state::AppState { db_pool: pool };

    let app = Router::new()
        .route("/", get(|| async { "请联系管理员获取服务" }))
        .merge(controller::oncall::router())
        .with_state(state.clone())
        .layer(cors);
    let addr = SocketAddr::from(([0, 0, 0, 0], 80));
    // 创建定时任务
    let db_pool = state.db_pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let res = controller::oncall::check_task_status(&db_pool).await;
            // 打印当前utc+8 时间
            let now =
                chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(8 * 3600).unwrap());
            println!("【{}】下列任务超时，已重置为待处理状态: {:?}", now, res);
        }
    });
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
