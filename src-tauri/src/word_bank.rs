use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::QueryBuilder;
use sqlx::Row;
use sqlx::SqlitePool;
use tauri::Manager;

const REQUIRED_HEADERS: [&str; 7] = [
    "word",
    "phonetic",
    "part_of_speech_and_meanings",
    "example_sentence",
    "example_translation",
    "audio_uk",
    "audio_us",
];

#[derive(Debug, Deserialize)]
struct FourRankCsvRecord {
    word: String,
    phonetic: Option<String>,
    part_of_speech_and_meanings: Option<String>,
    example_sentence: Option<String>,
    example_translation: Option<String>,
    audio_uk: Option<String>,
    audio_us: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ImportSummary {
    pub total: u64,
    pub upserted: u64,
    pub skipped: u64,
}

#[derive(Debug, Serialize)]
pub struct WordListCard {
    pub id: i64,
    pub name: String,
    pub word_count: i64,
    pub is_active: bool,
}

fn validate_headers(headers: &csv::StringRecord) -> Result<()> {
    let normalized: Vec<String> = headers
        .iter()
        .map(|header| header.trim_start_matches('\u{feff}').trim().to_string())
        .collect();

    for expected in REQUIRED_HEADERS {
        if !normalized.iter().any(|header| header == expected) {
            bail!("CSV 缺少必需列: {expected}");
        }
    }

    Ok(())
}

async fn open_pool(app: &tauri::AppHandle) -> Result<SqlitePool> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .context("解析应用数据目录失败")?;
    fs::create_dir_all(&app_data_dir).with_context(|| {
        format!("创建应用数据目录失败: {}", app_data_dir.display())
    })?;

    let db_path = app_data_dir.join("word-card.sqlite3");
    let options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .with_context(|| format!("Failed to open SQLite database {}", db_path.display()))?;

    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await
        .context("Failed to enable SQLite foreign keys")?;

    Ok(pool)
}

async fn ensure_schema(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
CREATE TABLE IF NOT EXISTS word_list (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
)
"#,
    )
    .execute(pool)
    .await
    .context("Failed to initialize word_list table")?;

    sqlx::query(
        r#"
CREATE TABLE IF NOT EXISTS word (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  word TEXT NOT NULL UNIQUE,
  phonetic TEXT,
  part_of_speech_and_meanings TEXT,
  example_sentence TEXT,
  example_translation TEXT,
  audio_uk TEXT,
  audio_us TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now'))
)
"#,
    )
    .execute(pool)
    .await
    .context("Failed to initialize word table")?;

    sqlx::query(
        r#"
CREATE TABLE IF NOT EXISTS word_list_map (
  word_list_id INTEGER NOT NULL,
  word_id INTEGER NOT NULL,
  PRIMARY KEY (word_list_id, word_id),
  FOREIGN KEY (word_list_id) REFERENCES word_list(id),
  FOREIGN KEY (word_id) REFERENCES word(id)
)
"#,
    )
    .execute(pool)
    .await
    .context("Failed to initialize word_list_map table")?;

    sqlx::query(
        r#"
CREATE TABLE IF NOT EXISTS word_list_state (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  active_word_list_id INTEGER,
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
)
"#,
    )
    .execute(pool)
    .await
    .context("Failed to initialize word_list_state table")?;

    sqlx::query(
        "INSERT OR IGNORE INTO word_list_state (id, active_word_list_id) VALUES (1, NULL)",
    )
    .execute(pool)
    .await
    .context("Failed to initialize word_list_state row")?;

    Ok(())
}

pub async fn init_database(app: &tauri::AppHandle) -> Result<()> {
    let pool = open_pool(app).await?;
    ensure_schema(&pool).await?;
    Ok(())
}

pub async fn list_word_lists(app: &tauri::AppHandle) -> Result<Vec<WordListCard>> {
    let pool = open_pool(app).await?;
    ensure_schema(&pool).await?;

    let rows = sqlx::query(
        r#"
SELECT
  wl.id AS id,
  wl.name AS name,
  COUNT(wlm.word_id) AS word_count,
  CASE
    WHEN wls.active_word_list_id = wl.id THEN 1
    ELSE 0
  END AS is_active
FROM word_list wl
LEFT JOIN word_list_map wlm ON wl.id = wlm.word_list_id
LEFT JOIN word_list_state wls ON wls.id = 1
GROUP BY wl.id
ORDER BY is_active DESC, wl.created_at DESC, wl.id DESC
"#,
    )
    .fetch_all(&pool)
    .await
    .context("读取词库列表失败")?;

    let mut lists = Vec::with_capacity(rows.len());
    for row in rows {
        let id: i64 = row.try_get("id").context("读取词库 ID 失败")?;
        let name: String = row.try_get("name").context("读取词库名称失败")?;
        let word_count: i64 = row
            .try_get("word_count")
            .context("读取词库单词数量失败")?;
        let is_active: i64 = row
            .try_get("is_active")
            .context("读取词库激活状态失败")?;
        lists.push(WordListCard {
            id,
            name,
            word_count,
            is_active: is_active != 0,
        });
    }
    Ok(lists)
}

pub async fn create_word_list(app: &tauri::AppHandle, name: &str) -> Result<i64> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        bail!("词库名称不能为空");
    }

    let pool = open_pool(app).await?;
    ensure_schema(&pool).await?;

    let existing: Option<i64> = sqlx::query_scalar("SELECT id FROM word_list WHERE name = ?")
        .bind(trimmed)
        .fetch_optional(&pool)
        .await
        .context("检查词库名称失败")?;
    if existing.is_some() {
        bail!("词库名称已存在");
    }

    let result = sqlx::query("INSERT INTO word_list (name) VALUES (?)")
        .bind(trimmed)
        .execute(&pool)
        .await
        .context("创建词库失败")?;
    Ok(result.last_insert_rowid())
}

pub async fn set_active_word_list(app: &tauri::AppHandle, word_list_id: i64) -> Result<()> {
    if word_list_id <= 0 {
        bail!("词库不存在");
    }

    let pool = open_pool(app).await?;
    ensure_schema(&pool).await?;

    let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM word_list WHERE id = ?")
        .bind(word_list_id)
        .fetch_optional(&pool)
        .await
        .context("检查词库是否存在失败")?;
    if exists.is_none() {
        bail!("词库不存在");
    }

    sqlx::query(
        "UPDATE word_list_state SET active_word_list_id = ?, updated_at = datetime('now') WHERE id = 1",
    )
    .bind(word_list_id)
    .execute(&pool)
    .await
    .context("更新当前词库失败")?;
    Ok(())
}

pub async fn clear_active_word_list(app: &tauri::AppHandle) -> Result<()> {
    let pool = open_pool(app).await?;
    ensure_schema(&pool).await?;

    sqlx::query(
        "UPDATE word_list_state SET active_word_list_id = NULL, updated_at = datetime('now') WHERE id = 1",
    )
    .execute(&pool)
    .await
    .context("清除当前词库失败")?;
    Ok(())
}

pub async fn delete_word_list(app: &tauri::AppHandle, word_list_id: i64) -> Result<()> {
    if word_list_id <= 0 {
        bail!("词库不存在");
    }

    let pool = open_pool(app).await?;
    ensure_schema(&pool).await?;

    let mut tx = pool.begin().await.context("开启数据库事务失败")?;
    let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM word_list WHERE id = ?")
        .bind(word_list_id)
        .fetch_optional(&mut *tx)
        .await
        .context("检查词库是否存在失败")?;
    if exists.is_none() {
        bail!("词库不存在");
    }

    sqlx::query("DELETE FROM word_list_map WHERE word_list_id = ?")
        .bind(word_list_id)
        .execute(&mut *tx)
        .await
        .context("删除词库关联失败")?;

    sqlx::query("DELETE FROM word_list WHERE id = ?")
        .bind(word_list_id)
        .execute(&mut *tx)
        .await
        .context("删除词库失败")?;

    sqlx::query(
        "UPDATE word_list_state SET active_word_list_id = NULL, updated_at = datetime('now') WHERE id = 1 AND active_word_list_id = ?",
    )
    .bind(word_list_id)
    .execute(&mut *tx)
    .await
    .context("更新当前词库失败")?;

    tx.commit().await.context("提交数据库事务失败")?;
    Ok(())
}

async fn import_four_rank_csv_internal(
    app: &tauri::AppHandle,
    csv_content: &str,
    word_list_id: Option<i64>,
) -> Result<ImportSummary> {
    let pool = open_pool(app).await?;
    ensure_schema(&pool).await?;

    if let Some(list_id) = word_list_id {
        let exists: Option<i64> =
            sqlx::query_scalar("SELECT id FROM word_list WHERE id = ?")
                .bind(list_id)
                .fetch_optional(&pool)
                .await
                .context("检查词库是否存在失败")?;
        if exists.is_none() {
            bail!("词库不存在");
        }
    }

    let csv_content = csv_content.trim_start_matches('\u{feff}');
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_content.as_bytes());

    let headers = reader.headers().context("读取 CSV 表头失败")?.clone();
    validate_headers(&headers)?;

    let mut tx = pool.begin().await.context("开启数据库事务失败")?;
    let mut total = 0u64;
    let mut upserted = 0u64;
    let mut skipped = 0u64;

    for row in reader.deserialize::<FourRankCsvRecord>() {
        total += 1;
        let record = match row {
            Ok(record) => record,
            Err(_) => {
                skipped += 1;
                continue;
            }
        };

        let word = record.word.trim();
        if word.is_empty() {
            skipped += 1;
            continue;
        }

        sqlx::query(
            r#"
INSERT INTO word (
  word,
  phonetic,
  part_of_speech_and_meanings,
  example_sentence,
  example_translation,
  audio_uk,
  audio_us
)
VALUES (?, ?, ?, ?, ?, ?, ?)
ON CONFLICT(word) DO UPDATE SET
  phonetic = excluded.phonetic,
  part_of_speech_and_meanings = excluded.part_of_speech_and_meanings,
  example_sentence = excluded.example_sentence,
  example_translation = excluded.example_translation,
  audio_uk = excluded.audio_uk,
  audio_us = excluded.audio_us
"#,
        )
        .bind(word)
        .bind(record.phonetic)
        .bind(record.part_of_speech_and_meanings)
        .bind(record.example_sentence)
        .bind(record.example_translation)
        .bind(record.audio_uk)
        .bind(record.audio_us)
        .execute(&mut *tx)
        .await
        .context("Failed to write word table")?;
        upserted += 1;

        if let Some(list_id) = word_list_id {
            let word_id: i64 = sqlx::query_scalar("SELECT id FROM word WHERE word = ?")
                .bind(word)
                .fetch_one(&mut *tx)
                .await
                .context("读取单词 ID 失败")?;
            sqlx::query(
                "INSERT OR IGNORE INTO word_list_map (word_list_id, word_id) VALUES (?, ?)",
            )
            .bind(list_id)
            .bind(word_id)
            .execute(&mut *tx)
            .await
            .context("写入词库关联失败")?;
        }
    }

    tx.commit().await.context("提交数据库事务失败")?;

    Ok(ImportSummary {
        total,
        upserted,
        skipped,
    })
}

pub async fn import_four_rank_csv(
    app: &tauri::AppHandle,
    csv_content: &str,
) -> Result<ImportSummary> {
    import_four_rank_csv_internal(app, csv_content, None).await
}

pub async fn import_four_rank_csv_file_with_list(
    app: &tauri::AppHandle,
    file_path: &Path,
    word_list_id: i64,
) -> Result<ImportSummary> {
    let csv_content = fs::read_to_string(file_path)
        .with_context(|| format!("读取 CSV 文件失败: {}", file_path.display()))?;
    import_four_rank_csv_internal(app, &csv_content, Some(word_list_id)).await
}
