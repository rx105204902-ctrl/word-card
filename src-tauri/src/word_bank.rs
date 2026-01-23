use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

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

#[derive(Debug, Serialize)]
pub struct LearningWord {
    pub id: i64,
    pub word: String,
    pub phonetic: Option<String>,
    pub part_of_speech_and_meanings: Option<String>,
    pub example_sentence: Option<String>,
    pub example_translation: Option<String>,
    pub audio_uk: Option<String>,
    pub audio_us: Option<String>,
    pub proficiency_score: i64,
}

#[derive(Debug, Serialize)]
pub struct LearningProgress {
    pub word_id: i64,
    pub proficiency_score: i64,
    pub learn_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DailyStudyCount {
    pub date: String,
    pub word_count: i64,
}

#[derive(Default)]
pub struct StudyCalendarCache {
    counts: Mutex<Option<Vec<DailyStudyCount>>>,
}

impl StudyCalendarCache {
    fn get(&self) -> Option<Vec<DailyStudyCount>> {
        self.counts.lock().ok().and_then(|cache| cache.clone())
    }

    fn set(&self, counts: Vec<DailyStudyCount>) {
        if let Ok(mut cache) = self.counts.lock() {
            *cache = Some(counts);
        }
    }

    pub fn invalidate(&self) {
        if let Ok(mut cache) = self.counts.lock() {
            *cache = None;
        }
    }
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
        .create_if_missing(true)
        .busy_timeout(Duration::from_secs(3));

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

    sqlx::query(
        r#"
CREATE TABLE IF NOT EXISTS user_word_learning (
  word_id INTEGER PRIMARY KEY,
  proficiency_score INTEGER NOT NULL DEFAULT 0,
  last_learned_at TEXT,
  learn_count INTEGER NOT NULL DEFAULT 0,
  FOREIGN KEY (word_id) REFERENCES word(id)
)
"#,
    )
    .execute(pool)
    .await
    .context("Failed to initialize user_word_learning table")?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_user_word_learning_score ON user_word_learning(proficiency_score)",
    )
    .execute(pool)
    .await
    .context("Failed to initialize user_word_learning indexes")?;

    sqlx::query(
        r#"
CREATE TABLE IF NOT EXISTS study_log (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  word_id INTEGER NOT NULL,
  learned_at TEXT NOT NULL DEFAULT (datetime('now')),
  FOREIGN KEY (word_id) REFERENCES word(id)
)
"#,
    )
    .execute(pool)
    .await
    .context("Failed to initialize study_log table")?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_study_log_learned_at ON study_log(learned_at)",
    )
    .execute(pool)
    .await
    .context("Failed to initialize study_log index")?;

    ensure_learning_columns(pool).await?;

    Ok(())
}

async fn ensure_learning_columns(pool: &SqlitePool) -> Result<()> {
    let rows = sqlx::query("PRAGMA table_info(user_word_learning)")
        .fetch_all(pool)
        .await
        .context("Failed to read user_word_learning schema")?;

    let mut columns = HashSet::new();
    for row in rows {
        let name: String = row
            .try_get("name")
            .context("Failed to read user_word_learning column name")?;
        columns.insert(name);
    }

    let has_last_studied = columns.contains("last_studied_at");
    let has_study_count = columns.contains("study_count");

    if !columns.contains("last_learned_at") {
        sqlx::query("ALTER TABLE user_word_learning ADD COLUMN last_learned_at TEXT")
            .execute(pool)
            .await
            .context("Failed to add last_learned_at column")?;
    }

    if !columns.contains("learn_count") {
        sqlx::query(
            "ALTER TABLE user_word_learning ADD COLUMN learn_count INTEGER NOT NULL DEFAULT 0",
        )
        .execute(pool)
        .await
        .context("Failed to add learn_count column")?;
    }

    if has_last_studied {
        sqlx::query(
            r#"
UPDATE user_word_learning
SET last_learned_at = COALESCE(last_learned_at, last_studied_at)
WHERE last_studied_at IS NOT NULL
"#,
        )
        .execute(pool)
        .await
        .context("Failed to migrate last_studied_at")?;
    }

    if has_study_count {
        sqlx::query(
            r#"
UPDATE user_word_learning
SET learn_count = CASE
  WHEN learn_count IS NULL OR learn_count = 0 THEN study_count
  ELSE learn_count
END
WHERE study_count IS NOT NULL
"#,
        )
        .execute(pool)
        .await
        .context("Failed to migrate study_count")?;
    }

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

async fn fetch_active_word_list_id(pool: &SqlitePool) -> Result<i64> {
    let active: Option<i64> =
        sqlx::query_scalar("SELECT active_word_list_id FROM word_list_state WHERE id = 1")
            .fetch_optional(pool)
            .await
            .context("Failed to load active word list")?;
    match active {
        Some(id) if id > 0 => Ok(id),
        _ => bail!("No active word list selected."),
    }
}

fn row_to_learning_word(row: sqlx::sqlite::SqliteRow) -> Result<LearningWord> {
    Ok(LearningWord {
        id: row.try_get("id").context("Failed to read word id")?,
        word: row.try_get("word").context("Failed to read word")?,
        phonetic: row
            .try_get("phonetic")
            .context("Failed to read phonetic")?,
        part_of_speech_and_meanings: row
            .try_get("part_of_speech_and_meanings")
            .context("Failed to read meanings")?,
        example_sentence: row
            .try_get("example_sentence")
            .context("Failed to read example sentence")?,
        example_translation: row
            .try_get("example_translation")
            .context("Failed to read example translation")?,
        audio_uk: row.try_get("audio_uk").context("Failed to read audio_uk")?,
        audio_us: row.try_get("audio_us").context("Failed to read audio_us")?,
        proficiency_score: row
            .try_get("proficiency_score")
            .context("Failed to read proficiency score")?,
    })
}

async fn fetch_words_with_condition(
    pool: &SqlitePool,
    word_list_id: i64,
    condition: &str,
    exclude_ids: &[i64],
    limit: i64,
) -> Result<Vec<LearningWord>> {
    if limit <= 0 {
        return Ok(Vec::new());
    }
    let mut builder = QueryBuilder::new(
        r#"
SELECT
  w.id AS id,
  w.word AS word,
  w.phonetic AS phonetic,
  w.part_of_speech_and_meanings AS part_of_speech_and_meanings,
  w.example_sentence AS example_sentence,
  w.example_translation AS example_translation,
  w.audio_uk AS audio_uk,
  w.audio_us AS audio_us,
  COALESCE(uwl.proficiency_score, 0) AS proficiency_score
FROM word w
JOIN word_list_map wlm ON w.id = wlm.word_id
LEFT JOIN user_word_learning uwl ON w.id = uwl.word_id
WHERE wlm.word_list_id = "#,
    );
    builder.push_bind(word_list_id);
    if !condition.trim().is_empty() {
        builder.push(" AND ");
        builder.push(condition);
    }
    if !exclude_ids.is_empty() {
        builder.push(" AND w.id NOT IN (");
        let mut separated = builder.separated(", ");
        for id in exclude_ids {
            separated.push_bind(id);
        }
        builder.push(")");
    }
    builder.push(" ORDER BY RANDOM() LIMIT ");
    builder.push_bind(limit);

    let rows = builder
        .build()
        .fetch_all(pool)
        .await
        .context("Failed to load learning words")?;

    rows.into_iter().map(row_to_learning_word).collect()
}

async fn allocate_learning_session_for_list(
    pool: &SqlitePool,
    word_list_id: i64,
) -> Result<Vec<LearningWord>> {
    let mut selected = Vec::new();
    let mut selected_ids = Vec::new();

    let unlearned = fetch_words_with_condition(
        pool,
        word_list_id,
        "uwl.word_id IS NULL",
        &selected_ids,
        20,
    )
    .await?;
    for word in unlearned {
        selected_ids.push(word.id);
        selected.push(word);
    }

    let low = fetch_words_with_condition(
        pool,
        word_list_id,
        "uwl.word_id IS NOT NULL AND uwl.proficiency_score < 4",
        &selected_ids,
        20,
    )
    .await?;
    for word in low {
        selected_ids.push(word.id);
        selected.push(word);
    }

    let mid = fetch_words_with_condition(
        pool,
        word_list_id,
        "uwl.word_id IS NOT NULL AND uwl.proficiency_score BETWEEN 4 AND 8",
        &selected_ids,
        6,
    )
    .await?;
    for word in mid {
        selected_ids.push(word.id);
        selected.push(word);
    }

    let high = fetch_words_with_condition(
        pool,
        word_list_id,
        "uwl.word_id IS NOT NULL AND uwl.proficiency_score BETWEEN 8 AND 10",
        &selected_ids,
        4,
    )
    .await?;
    for word in high {
        selected_ids.push(word.id);
        selected.push(word);
    }

    let remaining = 50_i64.saturating_sub(selected.len() as i64);
    if remaining > 0 {
        let fill =
            fetch_words_with_condition(pool, word_list_id, "1 = 1", &selected_ids, remaining)
                .await?;
        for word in fill {
            selected_ids.push(word.id);
            selected.push(word);
        }
    }

    if selected.is_empty() {
        bail!("No words available in the active list.");
    }

    Ok(selected)
}

pub async fn allocate_learning_session(app: &tauri::AppHandle) -> Result<Vec<LearningWord>> {
    let pool = open_pool(app).await?;
    ensure_schema(&pool).await?;
    let word_list_id = fetch_active_word_list_id(&pool).await?;
    allocate_learning_session_for_list(&pool, word_list_id).await
}

async fn ensure_learning_row(pool: &SqlitePool, word_id: i64) -> Result<()> {
    let result = sqlx::query(
        r#"
INSERT OR IGNORE INTO user_word_learning (word_id, proficiency_score, last_learned_at, learn_count)
SELECT ?, 0, datetime('now'), 0
WHERE EXISTS (SELECT 1 FROM word WHERE id = ?)
"#,
    )
    .bind(word_id)
    .bind(word_id)
    .execute(pool)
    .await
    .context("Failed to initialize learning row")?;

    if result.rows_affected() == 0 {
        let exists: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM user_word_learning WHERE word_id = ?",
        )
        .bind(word_id)
        .fetch_optional(pool)
        .await
        .context("Failed to verify learning row")?;
        if exists.is_none() {
            bail!("当前单词不存在或已被删除");
        }
    }
    Ok(())
}

async fn read_learning_progress(pool: &SqlitePool, word_id: i64) -> Result<LearningProgress> {
    let row = sqlx::query(
        "SELECT proficiency_score, learn_count FROM user_word_learning WHERE word_id = ?",
    )
    .bind(word_id)
    .fetch_one(pool)
    .await
    .context("Failed to read learning progress")?;
    Ok(LearningProgress {
        word_id,
        proficiency_score: row
            .try_get("proficiency_score")
            .context("Failed to read proficiency score")?,
        learn_count: row
            .try_get("learn_count")
            .context("Failed to read learn count")?,
    })
}

async fn record_study_event(pool: &SqlitePool, word_id: i64) -> Result<()> {
    sqlx::query("INSERT INTO study_log (word_id, learned_at) VALUES (?, datetime('now'))")
        .bind(word_id)
        .execute(pool)
        .await
        .context("Failed to insert study log")?;
    Ok(())
}

async fn increment_proficiency_for_word(
    pool: &SqlitePool,
    word_id: i64,
) -> Result<LearningProgress> {
    ensure_learning_row(pool, word_id).await?;
    sqlx::query(
        r#"
UPDATE user_word_learning
SET proficiency_score = MIN(10, proficiency_score + 1),
    learn_count = learn_count + 1,
    last_learned_at = datetime('now')
WHERE word_id = ?
"#,
    )
    .bind(word_id)
    .execute(pool)
    .await
    .context("Failed to increment proficiency")?;
    record_study_event(pool, word_id).await?;
    read_learning_progress(pool, word_id).await
}

async fn decrement_proficiency_for_word(
    pool: &SqlitePool,
    word_id: i64,
) -> Result<LearningProgress> {
    ensure_learning_row(pool, word_id).await?;
    sqlx::query(
        r#"
UPDATE user_word_learning
SET proficiency_score = MAX(0, proficiency_score - 1),
    last_learned_at = datetime('now')
WHERE word_id = ?
"#,
    )
    .bind(word_id)
    .execute(pool)
    .await
    .context("Failed to decrement proficiency")?;
    record_study_event(pool, word_id).await?;
    read_learning_progress(pool, word_id).await
}

pub async fn increment_proficiency(
    app: &tauri::AppHandle,
    cache: &StudyCalendarCache,
    word_id: i64,
) -> Result<LearningProgress> {
    if word_id <= 0 {
        bail!("Invalid word id");
    }
    let pool = open_pool(app).await?;
    ensure_schema(&pool).await?;
    let progress = increment_proficiency_for_word(&pool, word_id).await?;
    cache.invalidate();
    Ok(progress)
}

pub async fn decrement_proficiency(
    app: &tauri::AppHandle,
    cache: &StudyCalendarCache,
    word_id: i64,
) -> Result<LearningProgress> {
    if word_id <= 0 {
        bail!("Invalid word id");
    }
    let pool = open_pool(app).await?;
    ensure_schema(&pool).await?;
    let progress = decrement_proficiency_for_word(&pool, word_id).await?;
    cache.invalidate();
    Ok(progress)
}

async fn list_daily_study_counts_internal(pool: &SqlitePool) -> Result<Vec<DailyStudyCount>> {
    let rows = sqlx::query(
        r#"
SELECT date(learned_at) AS date, COUNT(DISTINCT word_id) AS word_count
FROM study_log
GROUP BY date(learned_at)
ORDER BY date(learned_at) ASC
"#,
    )
    .fetch_all(pool)
    .await
    .context("Failed to read daily study counts")?;

    let mut counts = Vec::with_capacity(rows.len());
    for row in rows {
        let date: String = row.try_get("date").context("Failed to read date")?;
        let word_count: i64 = row
            .try_get("word_count")
            .context("Failed to read word count")?;
        counts.push(DailyStudyCount { date, word_count });
    }
    Ok(counts)
}

pub async fn list_daily_study_counts(
    app: &tauri::AppHandle,
    cache: &StudyCalendarCache,
) -> Result<Vec<DailyStudyCount>> {
    if let Some(cached) = cache.get() {
        return Ok(cached);
    }
    let pool = open_pool(app).await?;
    ensure_schema(&pool).await?;
    let counts = list_daily_study_counts_internal(&pool).await?;
    cache.set(counts.clone());
    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::async_runtime;

    async fn setup_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(":memory:")
                    .create_if_missing(true),
            )
            .await
            .expect("Failed to open test database");
        ensure_schema(&pool)
            .await
            .expect("Failed to ensure schema");
        pool
    }

    async fn insert_word(pool: &SqlitePool, word: &str) -> i64 {
        let result = sqlx::query(
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
VALUES (?, NULL, NULL, NULL, NULL, NULL, NULL)
"#,
        )
        .bind(word)
        .execute(pool)
        .await
        .expect("Failed to insert word");
        result.last_insert_rowid()
    }

    async fn map_word(pool: &SqlitePool, word_list_id: i64, word_id: i64) {
        sqlx::query("INSERT INTO word_list_map (word_list_id, word_id) VALUES (?, ?)")
            .bind(word_list_id)
            .bind(word_id)
            .execute(pool)
            .await
            .expect("Failed to map word");
    }

    #[test]
    fn proficiency_updates_are_bounded() {
        async_runtime::block_on(async {
            let pool = setup_pool().await;
            let word_id = insert_word(&pool, "alpha").await;
            ensure_learning_row(&pool, word_id)
                .await
                .expect("Failed to ensure learning row");

            let progress = increment_proficiency_for_word(&pool, word_id)
                .await
                .expect("Failed to increment");
            assert_eq!(progress.proficiency_score, 1);

            sqlx::query("UPDATE user_word_learning SET proficiency_score = 10 WHERE word_id = ?")
                .bind(word_id)
                .execute(&pool)
                .await
                .expect("Failed to update score");
            let progress = increment_proficiency_for_word(&pool, word_id)
                .await
                .expect("Failed to increment at max");
            assert_eq!(progress.proficiency_score, 10);

            sqlx::query("UPDATE user_word_learning SET proficiency_score = 0 WHERE word_id = ?")
                .bind(word_id)
                .execute(&pool)
                .await
                .expect("Failed to reset score");
            let progress = decrement_proficiency_for_word(&pool, word_id)
                .await
                .expect("Failed to decrement at min");
            assert_eq!(progress.proficiency_score, 0);
        });
    }

    #[test]
    fn session_allocation_respects_buckets_and_list() {
        async_runtime::block_on(async {
            let pool = setup_pool().await;
            let list_id = sqlx::query("INSERT INTO word_list (name) VALUES (?)")
                .bind("list-a")
                .execute(&pool)
                .await
                .expect("Failed to create list")
                .last_insert_rowid();
            let other_list_id = sqlx::query("INSERT INTO word_list (name) VALUES (?)")
                .bind("list-b")
                .execute(&pool)
                .await
                .expect("Failed to create list")
                .last_insert_rowid();

            let mut ids = Vec::new();
            for i in 0..50 {
                let word_id = insert_word(&pool, &format!("word_{i}")).await;
                map_word(&pool, list_id, word_id).await;
                ids.push(word_id);
            }
            for i in 0..6 {
                let word_id = insert_word(&pool, &format!("other_{i}")).await;
                map_word(&pool, other_list_id, word_id).await;
            }

            for word_id in &ids[20..40] {
                sqlx::query(
                    "INSERT INTO user_word_learning (word_id, proficiency_score, learn_count) VALUES (?, 2, 1)",
                )
                .bind(*word_id)
                .execute(&pool)
                .await
                .expect("Failed to insert low score");
            }
            for word_id in &ids[40..46] {
                sqlx::query(
                    "INSERT INTO user_word_learning (word_id, proficiency_score, learn_count) VALUES (?, 5, 1)",
                )
                .bind(*word_id)
                .execute(&pool)
                .await
                .expect("Failed to insert mid score");
            }
            for word_id in &ids[46..50] {
                sqlx::query(
                    "INSERT INTO user_word_learning (word_id, proficiency_score, learn_count) VALUES (?, 9, 1)",
                )
                .bind(*word_id)
                .execute(&pool)
                .await
                .expect("Failed to insert high score");
            }

            let session = allocate_learning_session_for_list(&pool, list_id)
                .await
                .expect("Failed to allocate session");
            assert_eq!(session.len(), 50);

            let mut unlearned = 0;
            let mut low = 0;
            let mut mid = 0;
            let mut high = 0;
            for word in session {
                let in_list: i64 = sqlx::query_scalar(
                    "SELECT COUNT(1) FROM word_list_map WHERE word_list_id = ? AND word_id = ?",
                )
                .bind(list_id)
                .bind(word.id)
                .fetch_one(&pool)
                .await
                .expect("Failed to check list mapping");
                assert_eq!(in_list, 1);

                let score: Option<i64> = sqlx::query_scalar(
                    "SELECT proficiency_score FROM user_word_learning WHERE word_id = ?",
                )
                .bind(word.id)
                .fetch_optional(&pool)
                .await
                .expect("Failed to read score");
                match score {
                    None => unlearned += 1,
                    Some(value) if value < 4 => low += 1,
                    Some(value) if value <= 8 => mid += 1,
                    Some(_) => high += 1,
                }
            }

            assert_eq!(unlearned, 20);
            assert_eq!(low, 20);
            assert_eq!(mid, 6);
            assert_eq!(high, 4);
        });
    }
}
