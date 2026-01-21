use std::fs;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
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

    Ok(())
}

pub async fn init_database(app: &tauri::AppHandle) -> Result<()> {
    let pool = open_pool(app).await?;
    ensure_schema(&pool).await?;
    Ok(())
}

pub async fn import_four_rank_csv(app: &tauri::AppHandle, csv_content: &str) -> Result<ImportSummary> {
    let pool = open_pool(app).await?;
    ensure_schema(&pool).await?;

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
    }

    tx.commit().await.context("提交数据库事务失败")?;

    Ok(ImportSummary {
        total,
        upserted,
        skipped,
    })
}
