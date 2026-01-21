use std::collections::{HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{bail, Context, Result};
use base64::engine::general_purpose;
use base64::Engine;
use serde::Serialize;
use tauri::Manager;

use crate::word_bank;

const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

#[derive(Default)]
pub struct UploadState {
    sessions: Mutex<HashMap<String, UploadSession>>,
}

struct UploadSession {
    size: u64,
    chunk_size: u64,
    total_chunks: u64,
    received_chunks: HashSet<u64>,
    chunk_hashes: HashMap<u64, String>,
    file_path: PathBuf,
    completed: bool,
}

#[derive(Serialize)]
pub struct UploadImportResult {
    pub upload_id: String,
    pub summary: word_bank::ImportSummary,
}

fn get_upload_dir(app: &tauri::AppHandle) -> Result<PathBuf> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .context("解析应用数据目录失败")?;
    let upload_dir = app_data_dir.join("uploads");
    fs::create_dir_all(&upload_dir)
        .with_context(|| format!("创建上传目录失败: {}", upload_dir.display()))?;
    Ok(upload_dir)
}

fn validate_file(file_name: &str, size: u64) -> Result<()> {
    if !file_name.to_lowercase().ends_with(".csv") {
        bail!("仅支持 CSV 文件");
    }
    if size > MAX_FILE_SIZE {
        bail!("文件大小超过 100MB");
    }
    Ok(())
}

pub fn start_upload(
    app: &tauri::AppHandle,
    state: &UploadState,
    upload_id: &str,
    file_name: &str,
    size: u64,
    chunk_size: u64,
    total_chunks: u64,
) -> Result<()> {
    if upload_id.trim().is_empty() {
        bail!("上传 ID 不能为空");
    }
    validate_file(file_name, size)?;
    if chunk_size == 0 {
        bail!("分片大小不能为空");
    }
    let expected_chunks = (size + chunk_size - 1) / chunk_size;
    if expected_chunks != total_chunks {
        bail!("分片数量不匹配");
    }

    let upload_dir = get_upload_dir(app)?;
    let file_path = upload_dir.join(format!("{upload_id}.csv"));
    if file_path.exists() {
        bail!("上传任务已存在");
    }

    {
        let sessions = state
            .sessions
            .lock()
            .map_err(|_| anyhow::anyhow!("上传状态不可用"))?;
        if sessions.contains_key(upload_id) {
            bail!("上传任务已存在");
        }
    }

    let file = OpenOptions::new()
        .create_new(true)
        .read(true)
        .write(true)
        .open(&file_path)
        .with_context(|| format!("创建上传文件失败: {}", file_path.display()))?;
    file.set_len(size)
        .with_context(|| format!("预分配文件失败: {}", file_path.display()))?;

    let session = UploadSession {
        size,
        chunk_size,
        total_chunks,
        received_chunks: HashSet::new(),
        chunk_hashes: HashMap::new(),
        file_path,
        completed: false,
    };

    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| anyhow::anyhow!("上传状态不可用"))?;
    sessions.insert(upload_id.to_string(), session);
    Ok(())
}

pub fn upload_chunk(
    state: &UploadState,
    upload_id: &str,
    chunk_index: u64,
    total_chunks: u64,
    chunk_hash: &str,
    chunk_data_base64: &str,
) -> Result<()> {
    let (file_path, size, chunk_size, expected_total, existing_hash) = {
        let sessions = state
            .sessions
            .lock()
            .map_err(|_| anyhow::anyhow!("上传状态不可用"))?;
        let session = sessions
            .get(upload_id)
            .ok_or_else(|| anyhow::anyhow!("上传任务不存在"))?;
        if session.completed {
            bail!("上传任务已完成");
        }
        if total_chunks != session.total_chunks {
            bail!("分片总数不一致");
        }
        let existing_hash = session.chunk_hashes.get(&chunk_index).cloned();
        (
            session.file_path.clone(),
            session.size,
            session.chunk_size,
            session.total_chunks,
            existing_hash,
        )
    };

    if chunk_index >= expected_total {
        bail!("分片索引越界");
    }

    if let Some(existing_hash) = existing_hash {
        if existing_hash == chunk_hash {
            return Ok(());
        }
        bail!("分片哈希不一致");
    }

    let chunk_bytes = general_purpose::STANDARD
        .decode(chunk_data_base64)
        .context("分片数据解码失败")?;
    let expected_len = if chunk_index + 1 == expected_total {
        size - chunk_size * (expected_total - 1)
    } else {
        chunk_size
    };
    if chunk_bytes.len() as u64 != expected_len {
        bail!("分片大小不匹配");
    }

    let mut file = OpenOptions::new()
        .write(true)
        .open(&file_path)
        .with_context(|| format!("写入分片失败: {}", file_path.display()))?;
    let offset = chunk_index * chunk_size;
    file.seek(SeekFrom::Start(offset))
        .context("定位分片失败")?;
    file.write_all(&chunk_bytes).context("写入分片失败")?;

    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| anyhow::anyhow!("上传状态不可用"))?;
    if let Some(session) = sessions.get_mut(upload_id) {
        session.received_chunks.insert(chunk_index);
        session
            .chunk_hashes
            .insert(chunk_index, chunk_hash.to_string());
    }
    Ok(())
}

pub fn finish_upload(state: &UploadState, upload_id: &str) -> Result<()> {
    let mut sessions = state
        .sessions
        .lock()
        .map_err(|_| anyhow::anyhow!("上传状态不可用"))?;
    let session = sessions
        .get_mut(upload_id)
        .ok_or_else(|| anyhow::anyhow!("上传任务不存在"))?;
    if session.received_chunks.len() as u64 != session.total_chunks {
        bail!("分片未上传完成");
    }
    session.completed = true;
    Ok(())
}

pub fn cancel_upload(state: &UploadState, upload_id: &str) -> Result<()> {
    let session = {
        let mut sessions = state
            .sessions
            .lock()
            .map_err(|_| anyhow::anyhow!("上传状态不可用"))?;
        sessions.remove(upload_id)
    };
    if let Some(session) = session {
        let _ = fs::remove_file(&session.file_path);
    }
    Ok(())
}

pub fn delete_upload(state: &UploadState, upload_id: &str) -> Result<()> {
    cancel_upload(state, upload_id)
}

pub async fn import_uploaded_files(
    app: &tauri::AppHandle,
    state: &UploadState,
    upload_ids: Vec<String>,
    word_list_id: i64,
) -> Result<Vec<UploadImportResult>> {
    let mut results = Vec::new();
    for upload_id in upload_ids {
        let file_path = {
            let sessions = state
                .sessions
                .lock()
                .map_err(|_| anyhow::anyhow!("上传状态不可用"))?;
            let session = sessions
                .get(&upload_id)
                .ok_or_else(|| anyhow::anyhow!("上传任务不存在"))?;
            if !session.completed {
                bail!("上传尚未完成");
            }
            session.file_path.clone()
        };

        let summary =
            word_bank::import_four_rank_csv_file_with_list(app, &file_path, word_list_id).await?;
        results.push(UploadImportResult {
            upload_id: upload_id.clone(),
            summary,
        });

        let session = {
            let mut sessions = state
                .sessions
                .lock()
                .map_err(|_| anyhow::anyhow!("上传状态不可用"))?;
            sessions.remove(&upload_id)
        };
        if let Some(session) = session {
            let _ = fs::remove_file(&session.file_path);
        }
    }
    Ok(results)
}
