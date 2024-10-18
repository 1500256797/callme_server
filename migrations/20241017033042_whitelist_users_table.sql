-- Add migration script here
CREATE TABLE IF NOT EXISTS whitelist_users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

-- 创建索引以提高查询性能
CREATE INDEX IF NOT EXISTS idx_whitelist_users_user_id ON whitelist_users(user_id);