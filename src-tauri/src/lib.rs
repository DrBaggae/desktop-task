use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub due_date: String,
    pub priority: String,
    pub status: String,
    pub completed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppData {
    pub tasks: Vec<Task>,
}

fn get_data_path(_app: &tauri::AppHandle) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(format!("{}/.local/share/desktop-task/tasks.json", home))
}

fn load_data(app: &tauri::AppHandle) -> AppData {
    let path = get_data_path(app);
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or(AppData { tasks: vec![] })
    } else {
        AppData { tasks: vec![] }
    }
}

fn save_data(app: &tauri::AppHandle, data: &AppData) {
    let path = get_data_path(app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let content = serde_json::to_string_pretty(data).unwrap();
    fs::write(path, content).ok();
}

#[tauri::command]
fn get_tasks(app: tauri::AppHandle) -> Vec<Task> {
    load_data(&app).tasks
}

#[tauri::command]
fn add_task(app: tauri::AppHandle, task: Task) {
    let mut data = load_data(&app);
    data.tasks.push(task);
    save_data(&app, &data);
}

#[tauri::command]
fn delete_task(app: tauri::AppHandle, id: String) {
    let mut data = load_data(&app);
    data.tasks.retain(|t| t.id != id);
    save_data(&app, &data);
}

#[tauri::command]
fn complete_task(app: tauri::AppHandle, id: String) {
    let mut data = load_data(&app);
    if let Some(task) = data.tasks.iter_mut().find(|t| t.id == id) {
        task.completed = true;
        task.status = "completed".to_string();
    }
    save_data(&app, &data);
}

#[tauri::command]
fn update_task(app: tauri::AppHandle, task: Task) {
    let mut data = load_data(&app);
    if let Some(t) = data.tasks.iter_mut().find(|t| t.id == task.id) {
        *t = task;
    }
    save_data(&app, &data);
}

#[tauri::command]
fn open_main_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        window.show().ok();
        window.set_focus().ok();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec![])))
        .setup(|app| {
            let autostart_manager = app.autolaunch();
            let _ = autostart_manager.enable();

            let tray_menu = tauri::menu::MenuBuilder::new(app)
                .text("open_widget", "Show Widget")
                .text("open_app", "Open App")
                .separator()
                .text("quit", "Quit")
                .build()?;

            tauri::tray::TrayIconBuilder::new()
                .menu(&tray_menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open_widget" => {
                        if let Some(window) = app.get_webview_window("widget") {
                            window.show().ok();
                            window.set_focus().ok();
                        }
                    }
                    "open_app" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().ok();
                            window.set_focus().ok();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_tasks,
            add_task,
            delete_task,
            complete_task,
            update_task,
            open_main_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}