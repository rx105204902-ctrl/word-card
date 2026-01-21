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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            if let Err(error) = tauri::async_runtime::block_on(word_bank::init_database(&app.handle())) {
                eprintln!("Failed to initialize database: {error}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, import_four_rank_csv])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
