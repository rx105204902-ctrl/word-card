mod file_upload;
mod word_bank;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn import_four_rank_csv(
    app: tauri::AppHandle,
    csv_content: String,
) -> Result<word_bank::ImportSummary, String> {
    word_bank::import_four_rank_csv(&app, &csv_content)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_word_lists(app: tauri::AppHandle) -> Result<Vec<word_bank::WordListCard>, String> {
    word_bank::list_word_lists(&app)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn create_word_list(app: tauri::AppHandle, name: String) -> Result<i64, String> {
    word_bank::create_word_list(&app, &name)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn set_active_word_list(app: tauri::AppHandle, word_list_id: i64) -> Result<(), String> {
    word_bank::set_active_word_list(&app, word_list_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn clear_active_word_list(app: tauri::AppHandle) -> Result<(), String> {
    word_bank::clear_active_word_list(&app)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn delete_word_list(app: tauri::AppHandle, word_list_id: i64) -> Result<(), String> {
    word_bank::delete_word_list(&app, word_list_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn start_upload(
    app: tauri::AppHandle,
    state: tauri::State<'_, file_upload::UploadState>,
    upload_id: String,
    file_name: String,
    size: u64,
    chunk_size: u64,
    total_chunks: u64,
) -> Result<(), String> {
    file_upload::start_upload(
        &app,
        state.inner(),
        &upload_id,
        &file_name,
        size,
        chunk_size,
        total_chunks,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn upload_chunk(
    state: tauri::State<'_, file_upload::UploadState>,
    upload_id: String,
    chunk_index: u64,
    total_chunks: u64,
    chunk_hash: String,
    chunk_data_base64: String,
) -> Result<(), String> {
    file_upload::upload_chunk(
        state.inner(),
        &upload_id,
        chunk_index,
        total_chunks,
        &chunk_hash,
        &chunk_data_base64,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn finish_upload(
    state: tauri::State<'_, file_upload::UploadState>,
    upload_id: String,
) -> Result<(), String> {
    file_upload::finish_upload(state.inner(), &upload_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn cancel_upload(
    state: tauri::State<'_, file_upload::UploadState>,
    upload_id: String,
) -> Result<(), String> {
    file_upload::cancel_upload(state.inner(), &upload_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn delete_upload(
    state: tauri::State<'_, file_upload::UploadState>,
    upload_id: String,
) -> Result<(), String> {
    file_upload::delete_upload(state.inner(), &upload_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn import_uploaded_files(
    app: tauri::AppHandle,
    state: tauri::State<'_, file_upload::UploadState>,
    upload_ids: Vec<String>,
    word_list_id: i64,
) -> Result<Vec<file_upload::UploadImportResult>, String> {
    file_upload::import_uploaded_files(&app, state.inner(), upload_ids, word_list_id)
        .await
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(file_upload::UploadState::default())
        .setup(|app| {
            if let Err(error) = tauri::async_runtime::block_on(word_bank::init_database(&app.handle())) {
                eprintln!("Failed to initialize database: {error}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            import_four_rank_csv,
            list_word_lists,
            create_word_list,
            set_active_word_list,
            clear_active_word_list,
            delete_word_list,
            start_upload,
            upload_chunk,
            finish_upload,
            cancel_upload,
            delete_upload,
            import_uploaded_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
