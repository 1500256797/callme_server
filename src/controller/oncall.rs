use crate::state::AppState;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
#[derive(Deserialize, Serialize, Debug)]
pub struct AddPhoneTaskReq {
    pub user_id: String,
    pub phone_number: String,
    pub notification_content: String,
}

#[derive(Serialize, Debug)]
pub struct AddPhoneTaskResp {
    pub success: bool,
    pub msg: String,
    pub task_id: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct GetPhoneTaskResp {
    pub success: bool,
    pub msg: String,
    pub task: Option<PhoneTask>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PhoneTask {
    pub id: Option<i64>,
    pub user_id: String,
    pub phone_number: String,
    pub notification_content: String,
}

fn validate_content(content: &str) -> (bool, String) {
    // 检查长度
    if content.len() < 10 || content.len() > 200 {
        return (false, "内容长度必须在10到200个字符之间".to_string());
    }

    (true, "".to_string())
}
pub async fn add_phone_task_handler(
    State(state): State<AppState>,
    Json(add_phone_task_req): Json<AddPhoneTaskReq>,
) -> Json<AddPhoneTaskResp> {
    // check params
    let (is_valid, msg) = validate_content(&add_phone_task_req.notification_content);
    if !is_valid {
        return Json(AddPhoneTaskResp {
            success: false,
            msg,
            task_id: None,
        });
    }
    if add_phone_task_req.user_id.is_empty()
        || add_phone_task_req.phone_number.is_empty()
        || add_phone_task_req.notification_content.is_empty()
        || add_phone_task_req.phone_number.len() != 11
    {
        return Json(AddPhoneTaskResp {
            success: false,
            msg: "参数错误".to_string(),
            task_id: None,
        });
    }
    // check user_id is in whitelist
    let user_id = add_phone_task_req.user_id.clone();
    let user_id_in_whitelist =
        sqlx::query!("SELECT * FROM whitelist_users WHERE user_id = ?", user_id)
            .fetch_one(&state.db_pool)
            .await;
    if user_id_in_whitelist.is_err() {
        return Json(AddPhoneTaskResp {
            success: false,
            msg: "您不在白名单中，请联系管理员".to_string(),
            task_id: None,
        });
    }
    // 查询最近10分钟内是否已经添加过任务 防止短时间内重复添加任务
    let result = sqlx::query!(
        "SELECT * FROM phone_tasks WHERE user_id = ? AND phone_number = ? AND notification_content = ? AND notification_status = 0 AND created_at > datetime('now', '-10 minute')",
        add_phone_task_req.user_id,
        add_phone_task_req.phone_number,
        add_phone_task_req.notification_content
    )
    .fetch_one(&state.db_pool)
    .await;
    if result.is_ok() {
        return Json(AddPhoneTaskResp {
            success: false,
            msg: "您的通知频率太快了，请10分钟后再创建电话通知任务".to_string(),
            task_id: None,
        });
    }
    // insert phone task
    let result = sqlx::query!(
        "INSERT INTO phone_tasks (user_id, phone_number, notification_content,notification_status) VALUES (?, ?, ?, ?)",
        add_phone_task_req.user_id,
        add_phone_task_req.phone_number,
        add_phone_task_req.notification_content,
        0
    )
    .execute(&state.db_pool)
    .await;
    if result.is_err() {
        return Json(AddPhoneTaskResp {
            success: false,
            msg: "添加电话通知任务失败".to_string(),
            task_id: None,
        });
    }

    let add_phone_task_resp = AddPhoneTaskResp {
        success: true,
        msg: "添加电话通知任务成功".to_string(),
        task_id: Some(result.unwrap().last_insert_rowid().to_string()),
    };
    Json(add_phone_task_resp)
}

pub async fn get_phone_task_handler(State(state): State<AppState>) -> Json<GetPhoneTaskResp> {
    match get_and_update_task(&state.db_pool).await {
        Ok(task) => Json(GetPhoneTaskResp {
            success: true,
            msg: "成功获取任务".to_string(),
            task: Some(task),
        }),
        Err(e) => Json(GetPhoneTaskResp {
            success: false,
            msg: e,
            task: None,
        }),
    }
}

async fn get_and_update_task(pool: &SqlitePool) -> Result<PhoneTask, String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // 获取一个待开始的任务
    let task = sqlx::query_as!(
        PhoneTask,
        r#"
        SELECT id, user_id, phone_number, notification_content
        FROM phone_tasks
        WHERE notification_status = 0
        ORDER BY created_at ASC
        LIMIT 1
        "#
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(task) = task {
        // 将任务状态更新为进行中
        sqlx::query!(
            r#"
            UPDATE phone_tasks
            SET notification_status = 1
            WHERE id = ?
            "#,
            task.id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        tx.commit().await.map_err(|e| e.to_string())?;
        Ok(task)
    } else {
        Err("没有待处理的任务".to_string())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FinishPhoneTaskReq {
    pub task_id: i64,
}

#[derive(Serialize, Debug)]
pub struct FinishPhoneTaskResp {
    pub success: bool,
    pub msg: String,
}

pub async fn finish_phone_task_handler(
    State(state): State<AppState>,
    Json(finish_phone_task_req): Json<FinishPhoneTaskReq>,
) -> Json<FinishPhoneTaskResp> {
    let result = update_task_status(&state.db_pool, finish_phone_task_req.task_id, 2)
        .await
        .map_err(|e| e.to_string());
    if result.is_err() {
        return Json(FinishPhoneTaskResp {
            success: false,
            msg: "任务状态更新失败 请重试".to_string(),
        });
    }

    Json(FinishPhoneTaskResp {
        success: true,
        msg: "任务状态更新成功".to_string(),
    })
}

// 添加一个更新任务状态的函数
pub async fn update_task_status(
    pool: &SqlitePool,
    task_id: i64,
    status: i32,
) -> Result<(), String> {
    sqlx::query!(
        r#"
        UPDATE phone_tasks
        SET notification_status = ?
        WHERE id = ?
        "#,
        status,
        task_id
    )
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

// 启动tokio定时任务 检查距离update_time 超过5分钟 但是状态仍然为1的任务
pub async fn check_task_status(pool: &SqlitePool) -> Result<Vec<i64>, String> {
    // 使用 SQLite 的 datetime 函数获取 5 分钟前的时间
    let tasks = sqlx::query_as!(
        PhoneTask,
        r#"
            SELECT id, user_id, phone_number, notification_content 
            FROM phone_tasks 
            WHERE notification_status = 1 
            AND datetime(updated_at, '+5 minutes') < datetime('now','localtime')
            "#
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    let mut task_ids = Vec::new();
    for task in tasks {
        // 将超时任务的状态重置为 0（待处理）
        task_ids.push(task.id.unwrap());
        update_task_status(pool, task.id.unwrap(), 0).await?;
    }
    Ok(task_ids)
}

// define router
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/addPhoneTask", post(add_phone_task_handler))
        .route("/getPhoneTask", get(get_phone_task_handler))
        .route("/finishPhoneTask", post(finish_phone_task_handler))
}
