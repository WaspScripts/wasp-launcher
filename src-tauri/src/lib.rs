use std::{
    collections::HashMap,
    env,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};

use serde_json::json;
use tauri::{Emitter, Manager, State};
use tauri_plugin_store::StoreExt;

#[derive(Default)]
struct ExecutablePaths {
    simba: PathBuf,
    runelite: PathBuf,
    osclient: PathBuf,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn set_executable_path(
    app: tauri::AppHandle,
    paths: State<'_, Mutex<ExecutablePaths>>,
    exe: String,
    path: String,
) {
    let mut paths = paths.lock().unwrap();
    match exe.as_str() {
        "simba" => paths.simba = PathBuf::from(path.clone()),
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
fn get_executable_path(paths: State<'_, Mutex<ExecutablePaths>>, exe: String) -> String {
    let paths = paths.lock().unwrap();
    match exe.as_str() {
        "simba" => paths.simba.to_str().unwrap().to_string(),
        "runelite" => paths.runelite.to_str().unwrap().to_string(),
        "osclient" => paths.osclient.to_str().unwrap().to_string(),
        _ => paths.simba.to_str().unwrap().to_string(),
    }
}

#[tauri::command]
async fn run_executable(
    paths: State<'_, Mutex<ExecutablePaths>>,
    exe: String,
    args: Vec<String>,
) -> Result<String, String> {
    let paths = paths.lock().unwrap();
    let path: PathBuf = match exe.as_str() {
        "simba" => paths.simba.clone(),
        "runelite" => paths.runelite.clone(),
        "osclient" => paths.osclient.clone(),
        _ => paths.simba.clone(),
    };

    Command::new(path)
        .args(args)
        .spawn()
        .map_err(|err| err.to_string())?;

    Ok("Process started successfully".to_string())
}

#[tauri::command]
fn start_server(app: tauri::AppHandle) {
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

fn handle_client(mut stream: TcpStream, app: tauri::AppHandle) -> bool {
    const HTML: &str = "<html>\r\n    <head><title>Auth Complete</title></head>\r\n    <body>\r\n        <h2>Authentication Complete</h2>\r\n        <p>You may now close this window and return to the app.</p>\r\n    </body>\r\n</html>";
    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", HTML.len(), HTML);

    let mut buffer = [0; 1024];
    if stream.read(&mut buffer).is_err() {
        return false;
    }

    let request = String::from_utf8_lossy(&buffer);
    let Some(request_line) = request.lines().next() else {
        return false;
    };

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return false;
    }

    let path = parts[1];
    let Some(q_idx) = path.find('?') else {
        return false;
    };

    let query = &path[q_idx + 1..];
    let params: HashMap<&str, &str> = query
        .split('&')
        .filter_map(|kv| {
            let mut split = kv.splitn(2, '=');
            Some((split.next()?, split.next().unwrap_or("")))
        })
        .collect();

    let code = params.get("code").map(|&s| s.to_string());
    let error = params.get("error").map(|&s| s.to_string());

    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();

    let payload = serde_json::json!({
        "code": code,
        "error": error
    });

    let _ = app
        .emit("oauth-callback", payload)
        .expect("Failed to ping the front-end!");
    true
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app: &mut tauri::App| {
            let settings = app.store("settings.json")?;

            let program_files_str: String = env::var("PROGRAMFILES(X86)").unwrap_or_else(|_| {
                app.path()
                    .app_local_data_dir()
                    .expect("Local Data Dir doesn't exist on this system")
                    .to_string_lossy()
                    .into_owned()
            });

            let program_files: PathBuf = PathBuf::from(program_files_str);

            let paths: serde_json::Value = settings.get("paths").unwrap_or_else(|| {
                let paths = json!({});
                settings.set("paths", paths.clone());
                paths
            });

            println!("{}", paths);

            app.manage(Mutex::new(ExecutablePaths {
                simba: paths
                    .get("simba")
                    .and_then(|p: &serde_json::Value| p.as_str().map(PathBuf::from))
                    .unwrap_or_else(|| {
                        app.path()
                            .app_local_data_dir()
                            .expect("App Local Data Dir doesn't exist on this system")
                            .join("Simba\\2000\\Simba64.exe")
                    }),
                runelite: paths
                    .get("runelite")
                    .and_then(|p: &serde_json::Value| p.as_str().map(PathBuf::from))
                    .unwrap_or_else(|| {
                        app.path()
                            .local_data_dir()
                            .expect("Local Data Dir doesn't exist on this system")
                            .join("RuneLite\\RuneLite.exe")
                    }),
                osclient: paths
                    .get("osclient")
                    .and_then(|p: &serde_json::Value| p.as_str().map(PathBuf::from))
                    .unwrap_or_else(|| {
                        program_files.join(
                            "Jagex Launcher\\Games\\Old School RuneScape\\Client\\osclient.exe",
                        )
                    }),
            }));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_executable_path,
            get_executable_path,
            run_executable,
            start_server
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
