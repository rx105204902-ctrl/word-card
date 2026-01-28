mod word_bank;

use tauri::{
    Manager,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn hide_main_window(app: tauri::AppHandle) -> Result<(), String> {
    minimize_window_to_tray(&app)
}

#[tauri::command]
fn set_tray_tooltip(app: tauri::AppHandle, word: String) -> Result<(), String> {
    let trimmed = word.trim();
    let tooltip = if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    };
    if let Some(tray) = app.tray_by_id("main") {
        tray
            .set_tooltip(tooltip)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn minimize_window_to_tray(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.minimize();
        window.hide().map_err(|error| error.to_string())?;
    }
    Ok(())
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
async fn import_dictionary_csv(
    app: tauri::AppHandle,
    name: String,
    csv_content: String,
    overwrite: bool,
) -> Result<word_bank::ImportSummary, String> {
    word_bank::import_dictionary_csv(&app, &name, &csv_content, overwrite)
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
async fn allocate_learning_session(
    app: tauri::AppHandle,
) -> Result<Vec<word_bank::LearningWord>, String> {
    word_bank::allocate_learning_session(&app)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn increment_proficiency(
    app: tauri::AppHandle,
    cache: tauri::State<'_, word_bank::StudyCalendarCache>,
    word_id: i64,
) -> Result<word_bank::LearningProgress, String> {
    word_bank::increment_proficiency(&app, cache.inner(), word_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn decrement_proficiency(
    app: tauri::AppHandle,
    cache: tauri::State<'_, word_bank::StudyCalendarCache>,
    word_id: i64,
) -> Result<word_bank::LearningProgress, String> {
    word_bank::decrement_proficiency(&app, cache.inner(), word_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_daily_study_counts(
    app: tauri::AppHandle,
    cache: tauri::State<'_, word_bank::StudyCalendarCache>,
) -> Result<Vec<word_bank::DailyStudyCount>, String> {
    word_bank::list_daily_study_counts(&app, cache.inner())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn list_fuzzy_words(
    app: tauri::AppHandle,
    sort: Option<String>,
) -> Result<Vec<word_bank::FuzzyWordItem>, String> {
    word_bank::list_fuzzy_words(&app, sort)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn clear_fuzzy_marks(
    app: tauri::AppHandle,
    word_ids: Vec<i64>,
) -> Result<(), String> {
    word_bank::clear_fuzzy_marks(&app, word_ids)
        .await
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(word_bank::StudyCalendarCache::default())
        .setup(|app| {
            if let Err(error) = tauri::async_runtime::block_on(word_bank::init_database(&app.handle())) {
                eprintln!("Failed to initialize database: {error}");
            }
            let minimize_to_tray =
                MenuItemBuilder::with_id("minimize-to-tray", "最小化到托盘").build(app)?;
            let quit_app = MenuItemBuilder::with_id("quit-app", "退出").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&minimize_to_tray, &quit_app])
                .build()?;
            let mut tray_builder = TrayIconBuilder::with_id("main").tooltip("word-card");
            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }
            let _tray = tray_builder
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "minimize-to-tray" => {
                        let _ = minimize_window_to_tray(app);
                    }
                    "quit-app" => {
                        app.exit(0);
                    }
                    _ => (),
                })
                .on_tray_icon_event(|tray, event| {
                    let app = tray.app_handle();
                    let Some(window) = app.get_webview_window("main") else {
                        return;
                    };
                    let show_window = || {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    };
                    match event {
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } => {
                            show_window();
                        }
                        TrayIconEvent::DoubleClick { .. } => {
                            show_window();
                        }
                        _ => {}
                    }
                })
                .build(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            hide_main_window,
            list_word_lists,
            create_word_list,
            import_dictionary_csv,
            set_active_word_list,
            clear_active_word_list,
            delete_word_list,
            allocate_learning_session,
            increment_proficiency,
            decrement_proficiency,
            list_daily_study_counts,
            list_fuzzy_words,
            clear_fuzzy_marks,
            set_tray_tooltip
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
