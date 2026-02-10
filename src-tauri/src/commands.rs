use std::{
    fs::{create_dir_all, metadata, remove_dir_all, remove_file, set_permissions, File},
    io::Write,
    net::TcpListener,
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};

use serde_json::json;
use tauri::{Manager, State};
use tauri_plugin_http::reqwest::Client;
use tauri_plugin_store::StoreExt;

use crate::{
    client::{bring_window_to_top, list_processes, WindowMatch},
    server::handle_client,
    simba::{ensure_simba_directories, read_plugins_version, run_simba, sync_plugins_repo},
    LauncherVariables,
};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
pub fn get_dev_mode(launcher_vars: State<'_, Mutex<LauncherVariables>>) -> bool {
    let launcher_vars = launcher_vars.lock().unwrap();
    launcher_vars.devmode
}

#[tauri::command]
pub fn set_dev_mode(
    app: tauri::AppHandle,
    launcher_vars: State<'_, Mutex<LauncherVariables>>,
    state: bool,
) {
    let mut launcher_vars = launcher_vars.lock().unwrap();
    launcher_vars.devmode = state;

    let store = app
        .store("settings.json")
        .expect("Failed to retrieve settings.json store!");
    store.set("devmode", state);
}

#[tauri::command]
pub fn get_dev_updates(launcher_vars: State<'_, Mutex<LauncherVariables>>) -> bool {
    let launcher_vars = launcher_vars.lock().unwrap();
    launcher_vars.dev_updates
}

#[tauri::command]
pub fn set_dev_updates(
    app: tauri::AppHandle,
    launcher_vars: State<'_, Mutex<LauncherVariables>>,
    state: bool,
) {
    let mut launcher_vars = launcher_vars.lock().unwrap();
    launcher_vars.dev_updates = state;

    let store = app
        .store("settings.json")
        .expect("Failed to retrieve settings.json store!");
    store.set("dev_updates", state);
}

#[tauri::command]
pub fn get_executable_path(
    launcher_vars: State<'_, Mutex<LauncherVariables>>,
    exe: String,
) -> String {
    let paths = launcher_vars.lock().unwrap();
    match exe.as_str() {
        "simba" => paths.simba.to_str().unwrap().to_string(),
        "devsimba" => paths.devsimba.to_str().unwrap().to_string(),
        "runelite" => paths.runelite.to_str().unwrap().to_string(),
        "osclient" => paths.osclient.to_str().unwrap().to_string(),
        _ => paths.simba.to_str().unwrap().to_string(),
    }
}

#[tauri::command]
pub fn set_executable_path(
    app: tauri::AppHandle,
    launcher_vars: State<'_, Mutex<LauncherVariables>>,
    exe: String,
    path: String,
) {
    let mut paths = launcher_vars.lock().unwrap();
    match exe.as_str() {
        "simba" => paths.simba = PathBuf::from(path.clone()),
        "devsimba" => paths.devsimba = PathBuf::from(path.clone()),
        "runelite" => paths.runelite = PathBuf::from(path.clone()),
        "osclient" => paths.osclient = PathBuf::from(path.clone()),
        _ => {}
    }

    let store = app
        .store("settings.json")
        .expect("Failed to retrieve settings.json store!");
    store.set("paths", json!({exe.as_str(): path}));
}

#[tauri::command]
pub fn delete_cache(
    launcher_vars: State<'_, Mutex<LauncherVariables>>,
    exe: String,
) -> tauri::Result<()> {
    let path = {
        let paths = launcher_vars.lock().unwrap();
        if exe == "devsimba" {
            paths.devsimba.clone()
        } else {
            paths.simba.clone()
        }
    };
    let cache_path = path.join("Data").join("Cache");

    if cache_path.exists() {
        remove_dir_all(&cache_path).expect("Failed to delete cache path.");
        println!("Deleted folder: {:?}", cache_path);
    }

    Ok(())
}

#[tauri::command]
pub fn delete_assets(
    launcher_vars: State<'_, Mutex<LauncherVariables>>,
    exe: String,
) -> tauri::Result<()> {
    let path = {
        let paths = launcher_vars.lock().unwrap();
        if exe == "devsimba" {
            paths.devsimba.clone()
        } else {
            paths.simba.clone()
        }
    };

    let assets_path = path.join("Data").join("Assets");

    if assets_path.exists() {
        remove_dir_all(&assets_path).expect("Failed to delete assets path.");
        println!("Deleted folder: {:?}", assets_path);
    }

    Ok(())
}

#[tauri::command]
pub fn delete_configs(
    launcher_vars: State<'_, Mutex<LauncherVariables>>,
    exe: String,
) -> tauri::Result<()> {
    let path = {
        let paths = launcher_vars.lock().unwrap();
        if exe == "devsimba" {
            paths.devsimba.clone()
        } else {
            paths.simba.clone()
        }
    };

    let configs = path.join("Configs");

    if configs.exists() {
        remove_dir_all(&configs).expect("Failed to delete configs path.");
        println!("Deleted folder: {:?}", configs);
    }

    Ok(())
}

#[tauri::command]
pub fn save_blob(
    app: tauri::AppHandle,
    path: String,
    filename: String,
    data: Vec<u8>,
) -> Result<(), String> {
    let file_path = app
        .path()
        .app_local_data_dir()
        .expect("App Local Data Dir doesn't exist on this system")
        .join("Simba")
        .join("Scripts")
        .join(path)
        .join(filename);

    if let Some(parent) = file_path.parent() {
        create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    if file_path.exists() {
        remove_file(&file_path).map_err(|e| e.to_string())?;
    }

    println!("File path: {:?}", file_path);

    let mut file = File::create(&file_path).map_err(|e| e.to_string())?;
    file.write_all(&data).map_err(|e| e.to_string())?;
    drop(file);

    let mut perms = metadata(&file_path)
        .map_err(|e| e.to_string())?
        .permissions();

    perms.set_readonly(true);
    set_permissions(&file_path, perms).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn run_executable(
    launcher_vars: State<'_, Mutex<LauncherVariables>>,
    exe: String,
    args: Vec<String>,
) -> Result<String, String> {
    let args_clone = args.clone();

    let path = {
        let paths = launcher_vars.lock().unwrap();
        match exe.as_str() {
            "simba" => paths.simba.clone(),
            "devsimba" => paths.devsimba.clone(),
            "runelite" => paths.runelite.clone(),
            "osclient" => paths.osclient.clone(),
            _ => paths.simba.clone(),
        }
    };

    if exe == "simba" {
        run_simba(path, args).await;
    } else if exe == "devsimba" {
        let diff_dirs = {
            let paths = launcher_vars.lock().unwrap();
            paths.simba != paths.devsimba
        };

        if diff_dirs {
            let _ = ensure_simba_directories(&path);
            let plugins_path = path.join("Plugins").join("wasp-plugins");
            tauri::async_runtime::spawn(async move {
                let _ = sync_plugins_repo(&plugins_path);
            });
        };

        run_simba(path, args).await;
    } else {
        Command::new(path)
            .args(args_clone)
            .spawn()
            .map_err(|err| err.to_string())?;
    }

    Ok("Process started successfully".to_string())
}

#[tauri::command]
pub fn start_server(app: tauri::AppHandle) {
    let Ok(listener) = TcpListener::bind("127.0.0.1:5217") else {
        return;
    };

    println!("Auth Server listening on localhost:5217");

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    thread::spawn(move || {
        for stream in listener
            .incoming()
            .take_while(|_| running_clone.load(Ordering::Relaxed))
        {
            match stream {
                Ok(stream) => {
                    let app_clone = app.clone();
                    if handle_client(stream, app_clone) {
                        running_clone.store(false, Ordering::Relaxed);
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        println!("Auth Server stopped!");
    });
}

#[tauri::command]
pub async fn sign_up(id: String) -> Result<String, String> {
    println!("Sign up for user {}", id);

    let client = Client::new();
    let url = "https://waspscripts.dev/auth/launcher/";

    let body = json!({
        "user_id": id
    });

    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request error: {}", e))
        .expect("Request error");

    let text = res
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))
        .expect("Failed to read response");

    Ok(text)
}

#[tauri::command]
pub fn get_plugin_version(
    launcher_vars: State<'_, Mutex<LauncherVariables>>,
) -> Result<String, String> {
    let path = {
        let paths = launcher_vars.lock().unwrap();
        paths.simba.clone()
    };
    let version_path = path.join("Plugins/wasp-plugins/version.simba");

    read_plugins_version(&version_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reinstall_plugins(
    launcher_vars: State<'_, Mutex<LauncherVariables>>,
    exe: String,
) -> tauri::Result<()> {
    let path = {
        let paths = launcher_vars.lock().unwrap();
        if exe == "devsimba" {
            paths.devsimba.clone()
        } else {
            paths.simba.clone()
        }
    };

    println!("Reinstalling plugins!");
    let plugins_path = path.join("Plugins").join("wasp-plugins");

    if plugins_path.exists() {
        remove_dir_all(&plugins_path).expect("Failed to delete wasp-plugins path.");
        println!("Deleted folder: {:?}", plugins_path);
    }

    let _ = sync_plugins_repo(&plugins_path).await;

    Ok(())
}

#[tauri::command]
pub async fn list_clients() -> Result<Vec<WindowMatch>, String> {
    list_processes()
}

#[tauri::command]
pub async fn show_client(hwnd: isize) -> Result<(), String> {
    if bring_window_to_top(hwnd) {
        Ok(())
    } else {
        Err("Failed to bring window to front. The handle might be invalid.".to_string())
    }
}
