use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use tauri::Listener;
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
        window.set_skip_taskbar(false).ok();
        window.show().ok();
        window.center().ok();
        window.set_focus().ok();
    }
}

fn handle_deep_link(app: &tauri::AppHandle, urls: Vec<url::Url>) {
    for url in urls {
        let path = url.path();
        let query = url.query().unwrap_or("");
        let id = query.split('=').nth(1).unwrap_or("").to_string();
        if path == "/complete" {
            complete_task(app.clone(), id);
        } else if path == "/delete" {
            delete_task(app.clone(), id);
        }
    }
}

fn start_command_watcher(app: tauri::AppHandle) {
    let home = std::env::var("HOME").unwrap_or_default();
    let command_path = PathBuf::from(format!("{}/.local/share/desktop-task/command.txt", home));

    std::thread::spawn(move || {
        let mut last_content = String::new();
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            if command_path.exists() {
                if let Ok(content) = fs::read_to_string(&command_path) {
                    let content = content.trim().to_string();
                    if !content.is_empty() && content != last_content {
                        last_content = content.clone();
                        println!("Command received: {}", content);

                        match content.as_str() {
                            "open-app" => {
                                std::thread::sleep(std::time::Duration::from_millis(100));
                                open_main_window(app.clone());
                            }
                            _ => {
                                let parts: Vec<&str> = content.splitn(2, ':').collect();
                                if parts.len() == 2 {
                                    let cmd = parts[0];
                                    let id = parts[1].to_string();
                                    match cmd {
                                        "complete" => complete_task(app.clone(), id),
                                        "delete" => delete_task(app.clone(), id),
                                        _ => {}
                                    }
                                }
                            }
                        }

                        fs::write(&command_path, "").ok();
                    }
                }
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // webkit2gtk-4.1 crashes on some Linux systems when using DMA-BUF renderer
    #[cfg(target_os = "linux")]
    unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1"); }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if let Some(url) = argv.get(1) {
                if let Ok(parsed) = url::Url::parse(url) {
                    handle_deep_link(app, vec![parsed]);
                    return;
                }
            }
            open_main_window(app.clone());
        }))
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            let autostart_manager = app.autolaunch();
            let _ = autostart_manager.enable();

            start_command_watcher(app.handle().clone());

            let main_win = app.get_webview_window("main").unwrap();
            let main_win_clone = main_win.clone();
            main_win.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    main_win_clone.set_skip_taskbar(true).ok();
                    main_win_clone.set_position(tauri::Position::Physical(
                        tauri::PhysicalPosition::new(-10000, -10000),
                    )).ok();
                }
            });

            let app_handle = app.handle().clone();
            app.listen("deep-link://new-url", move |event| {
                println!("Deep link received: {:?}", event.payload());
                if let Ok(urls) = serde_json::from_str::<Vec<String>>(event.payload()) {
                    let parsed: Vec<url::Url> = urls
                        .iter()
                        .filter_map(|u| url::Url::parse(u).ok())
                        .collect();
                    handle_deep_link(&app_handle, parsed);
                }
            });

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
                        open_main_window(app.clone());
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