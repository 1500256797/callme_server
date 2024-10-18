-- Add migration script here
CREATE TABLE IF NOT EXISTS phone_tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    phone_number TEXT NOT NULL,
    notification_content TEXT NOT NULL,
    notification_status INTEGER NOT NULL CHECK(notification_status IN (0, 1, 2)),
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

-- 创建触发器以自动更新 updated_at 字段
CREATE TRIGGER IF NOT EXISTS update_phone_tasks_timestamp 
AFTER UPDATE ON phone_tasks
FOR EACH ROW
BEGIN
    UPDATE phone_tasks SET updated_at = datetime('now', 'localtime') WHERE id = OLD.id;
END;

-- 创建索引以提高查询性能
CREATE INDEX IF NOT EXISTS idx_phone_tasks_user_id ON phone_tasks(user_id);
CREATE INDEX IF NOT EXISTS idx_phone_tasks_phone_number ON phone_tasks(phone_number);
CREATE INDEX IF NOT EXISTS idx_phone_tasks_notification_status ON phone_tasks(notification_status);