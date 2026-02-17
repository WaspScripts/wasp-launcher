use std::{
    fs::{create_dir_all, metadata, remove_dir_all, remove_file, set_permissions, File},
    io::Write,
    net::TcpListener,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};

use serde_json::json;
use tauri::{ipc::Channel, Emitter, Manager, State};
use tauri_plugin_http::reqwest::Client;
use tauri_plugin_store::StoreExt;

use crate::{
    client::{bring_window_to_top, list_processes, WindowMatch},
    server::handle_client,
    simba::{
        ensure_simba_directories, read_plugins_version, run_simba, run_simba_script,
        sync_plugins_repo,
    },
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
    launcher: State<'_, Mutex<LauncherVariables>>,
    state: bool,
) {
    let mut launcher = launcher.lock().unwrap();
    launcher.devmode = state;

    let store = app
        .store("settings.json")
        .expect("Failed to retrieve settings.json store!");
    store.set("devmode", state);
}

#[tauri::command]
pub fn get_dev_updates(launcher: State<'_, Mutex<LauncherVariables>>) -> bool {
    let launcher_vars = launcher.lock().unwrap();
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
pub fn get_executable_path(launcher: State<'_, Mutex<LauncherVariables>>, exe: String) -> String {
    let paths = launcher.lock().unwrap();
    match exe.as_str() {
        "simba" => paths.simba.to_str().unwrap().to_string(),
        "devsimba" => paths.devsimba.to_str().unwrap().to_string(),
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
    let path = {
        let paths = launcher_vars.lock().unwrap();
        match exe.as_str() {
            "simba" => paths.simba.clone(),
            "devsimba" => paths.devsimba.clone(),
            _ => paths.simba.clone(),
        }
    };

    if exe == "simba" {
        run_simba(path, args).await;
        Ok("Process started successfully".to_string())
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
        Ok("Process started successfully".to_string())
    } else {
        Err("Unrecognized executable. Only \"simba\" or \"devsimba\" is allowed.".to_string())
    }
}

#[tauri::command]
pub async fn run_script(
    app: tauri::AppHandle,
    launcher: State<'_, Mutex<LauncherVariables>>,
    args: Vec<String>,
    channel: Channel<String>,
) -> Result<String, String> {
    let (simba_path, hwnd) = {
        let guard = launcher.lock().unwrap();
        match &guard.client {
            Some(client) => (guard.simba.clone(), client.hwnd),
            None => return Err("Client is null".to_string()),
        }
    };

    let id = channel.id();
    let process = run_simba_script(simba_path, hwnd, args, channel).await?;

    let shared_process = Arc::new(Mutex::new(Some(process)));

    let guard = launcher.lock().unwrap();
    guard
        .scripts
        .lock()
        .unwrap()
        .insert(id, shared_process.clone());

    let app_clone = app.clone();

    std::thread::spawn(move || loop {
        let status = {
            let mut inner_guard = shared_process.lock().unwrap();
            if let Some(child) = inner_guard.as_mut() {
                child.try_wait()
            } else {
                return;
            }
        };

        match status {
            Ok(Some(exit_status)) => {
                println!("Process {} exited with status: {}", id, exit_status);

                if let Some(launcher_state) = app_clone.try_state::<Mutex<LauncherVariables>>() {
                    let guard = launcher_state.lock().unwrap();
                    guard.scripts.lock().unwrap().remove(&id);
                }

                let _ = app_clone.emit("process-finished", id);
                break;
            }
            Ok(None) => {
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
            Err(e) => {
                println!("Error checking process status: {}", e);
                break;
            }
        }
    });

    Ok("Process started successfully".to_string())
}

#[tauri::command]
pub async fn kill_script(
    app: tauri::AppHandle,
    launcher: tauri::State<'_, Mutex<LauncherVariables>>,
    id: u32,
) -> Result<String, String> {
    let handle = {
        let launcher_guard = launcher.lock().unwrap();
        let scripts_guard = launcher_guard.scripts.lock().unwrap();
        scripts_guard.get(&id).cloned()
    };

    if let Some(shared_process) = handle {
        let mut process_guard = shared_process.lock().unwrap();

        if let Some(mut child) = process_guard.take() {
            let result = child.kill().map_err(|e| e.to_string());

            let launcher_guard = launcher.lock().unwrap();
            launcher_guard.scripts.lock().unwrap().remove(&id);
            let _ = app.emit("process-finished", id);

            match result {
                Ok(_) => Ok(format!("Process {} killed", id)),
                Err(e) => Err(format!("Failed to kill: {}", e)),
            }
        } else {
            Err(format!("Process {} is already stopping or finished", id))
        }
    } else {
        Err(format!("No active script found for ID {}", id))
    }
}

#[tauri::command]
pub async fn get_running_scripts() -> Result<(), String> {
    Ok(())
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
        .body(body.to_string())
        .send()
        .await
        .map_err(|e| format!("Request error: {}", e))?;

    let text = res
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

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
pub async fn set_client(
    launcher: State<'_, Mutex<LauncherVariables>>,
    client: Option<WindowMatch>,
) -> tauri::Result<()> {
    let mut launcher = launcher.lock().unwrap();
    launcher.client = client;
    Ok(())
}

#[tauri::command]
pub async fn show_client(launcher: State<'_, Mutex<LauncherVariables>>) -> Result<(), String> {
    let launcher = launcher.lock().unwrap();
    match &launcher.client {
        Some(client) => {
            let hwnd = client.hwnd;
            if bring_window_to_top(hwnd) {
                Ok(())
            } else {
                Err("Failed to bring window to front. The handle might be invalid.".to_string())
            }
        }
        None => Err("Client is null".to_string()),
    }
}
