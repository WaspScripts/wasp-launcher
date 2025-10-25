use std::{
    fs::{remove_dir_all, File},
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
    server::handle_client, simba::ensure_simba_directories, simba::read_plugins_version,
    simba::run_simba, simba::sync_plugins_repo, ExecutablePaths,
};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
pub fn set_executable_path(
    app: tauri::AppHandle,
    paths: State<'_, Mutex<ExecutablePaths>>,
    exe: String,
    path: String,
) {
    let mut paths = paths.lock().unwrap();
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
pub fn get_executable_path(paths: State<'_, Mutex<ExecutablePaths>>, exe: String) -> String {
    let paths = paths.lock().unwrap();
    match exe.as_str() {
        "simba" => paths.simba.to_str().unwrap().to_string(),
        "devsimba" => paths.devsimba.to_str().unwrap().to_string(),
        "runelite" => paths.runelite.to_str().unwrap().to_string(),
        "osclient" => paths.osclient.to_str().unwrap().to_string(),
        _ => paths.simba.to_str().unwrap().to_string(),
    }
}

#[tauri::command]
pub fn delete_cache(paths: State<'_, Mutex<ExecutablePaths>>) -> tauri::Result<()> {
    let path = {
        let paths = paths.lock().unwrap();
        paths.simba.clone()
    };
    let cache_path = path.join("Data/Cache");

    if cache_path.exists() {
        remove_dir_all(&cache_path).expect("Failed to delete cache path.");
        println!("Deleted folder: {:?}", cache_path);
    }

    Ok(())
}

#[tauri::command]
pub fn delete_assets(paths: State<'_, Mutex<ExecutablePaths>>) -> tauri::Result<()> {
    let path = {
        let paths = paths.lock().unwrap();
        paths.simba.clone()
    };

    let assets_path = path.join("Data/Assets");

    if assets_path.exists() {
        remove_dir_all(&assets_path).expect("Failed to delete assets path.");
        println!("Deleted folder: {:?}", assets_path);
    }

    Ok(())
}

#[tauri::command]
pub fn delete_configs(paths: State<'_, Mutex<ExecutablePaths>>) -> tauri::Result<()> {
    let path = {
        let paths = paths.lock().unwrap();
        paths.simba.clone()
    };

    let assets_path = path.join("Configs");

    if assets_path.exists() {
        remove_dir_all(&assets_path).expect("Failed to delete configs path.");
        println!("Deleted folder: {:?}", assets_path);
    }

    Ok(())
}

#[tauri::command]
pub fn save_blob(app: tauri::AppHandle, path: String, data: Vec<u8>) -> Result<(), String> {
    let final_path = app
        .path()
        .app_local_data_dir()
        .expect("App Local Data Dir doesn't exist on this system")
        .join("Simba")
        .join("Scripts")
        .join(path);

    if final_path.exists() {
        std::fs::remove_file(&final_path).map_err(|e| e.to_string())?;
    }

    let mut file = File::create(&final_path).map_err(|e| e.to_string())?;
    file.write_all(&data).map_err(|e| e.to_string())?;
    drop(file);

    let mut perms = std::fs::metadata(&final_path)
        .map_err(|e| e.to_string())?
        .permissions();

    perms.set_readonly(true);
    std::fs::set_permissions(&final_path, perms).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn run_executable(
    paths: State<'_, Mutex<ExecutablePaths>>,
    exe: String,
    args: Vec<String>,
) -> Result<String, String> {
    let args_clone = args.clone();

    let path = {
        let paths = paths.lock().unwrap();
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
            let paths = paths.lock().unwrap();
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
pub fn get_plugin_version(paths: State<'_, Mutex<ExecutablePaths>>) -> Result<String, String> {
    let path = {
        let paths = paths.lock().unwrap();
        paths.simba.clone()
    };
    let version_path = path.join("Plugins/wasp-plugins/version.simba");

    read_plugins_version(&version_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reinstall_plugins(paths: State<'_, Mutex<ExecutablePaths>>) -> tauri::Result<()> {
    let path = {
        let paths = paths.lock().unwrap();
        paths.simba.clone()
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
