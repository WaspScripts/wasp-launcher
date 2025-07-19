use std::{
    collections::HashMap,
    env,
    fs::{create_dir_all, remove_file, File},
    io::{Cursor, Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};

use git2::{Cred, FetchOptions, RemoteCallbacks, Repository, StashFlags};
use serde_json::json;
use tauri::{Emitter, Manager, State};
use tauri_plugin_store::StoreExt;

use tauri_plugin_http::reqwest::{self, Client};
use zip::ZipArchive;

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

fn sync_plugins_repo(plugins_path: &PathBuf) -> Result<(), git2::Error> {
    let repo = match Repository::open(plugins_path) {
        Ok(mut repo) => {
            // Stash changes (if any)
            let index = repo.index()?;
            if index.has_conflicts() || !index.is_empty() {
                let sig = repo.signature()?;
                let _ = repo.stash_save(
                    &sig,
                    "Auto stash before pull",
                    Some(StashFlags::INCLUDE_UNTRACKED),
                );
            }
            repo
        }
        Err(_) => Repository::clone("https://github.com/WaspScripts/wasp-plugins", plugins_path)?,
    };

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, _username_from_url, _allowed_types| Cred::default());

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    // Pull from origin
    let mut remote = repo.find_remote("origin")?;
    remote.fetch(&["main"], Some(&mut fetch_opts), None)?;

    let fetch_head = repo.reference_to_annotated_commit(&repo.find_reference("FETCH_HEAD")?)?;
    let ref_heads = repo.find_branch("main", git2::BranchType::Local)?;
    let analysis = repo.merge_analysis(&[&fetch_head])?;

    if analysis.0.is_fast_forward() {
        let mut reference = ref_heads.into_reference();
        reference.set_target(fetch_head.id(), "Fast-forward")?;
        repo.set_head(reference.name().unwrap())?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
    } else {
        println!("Merge needed or nothing to do");
    }

    Ok(())
}

fn ensure_simba_directories(path: &PathBuf) -> std::io::Result<()> {
    create_dir_all(path)?;

    // List of subfolders to create
    let dirs = [
        "Configs",
        "Data",
        "Includes",
        "Plugins",
        "Screenshots",
        "Scripts",
    ];

    for dir in &dirs {
        let subdir = path.join(dir);
        create_dir_all(&subdir)?;
    }

    Ok(())
}

async fn download_and_unzip(
    url: &str,
    dest: &PathBuf, // Full path including filename like Simba-simba2000.exe
) -> Result<(), Box<dyn std::error::Error>> {
    // Download ZIP to memory
    let response = Client::new()
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let cursor = Cursor::new(response);
    let mut archive = ZipArchive::new(cursor)?;

    if archive.len() != 1 {
        return Err(format!("Expected 1 file in ZIP, found {}", archive.len()).into());
    }

    let mut file = archive.by_index(0)?;
    if file.name().ends_with('/') {
        return Err("Unexpected directory in zip".into());
    }

    // Ensure parent directory exists
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write the file using `dest` as the output path
    let mut out_file = File::create(dest)?;
    std::io::copy(&mut file, &mut out_file)?;

    Ok(())
}

async fn run_simba(path: PathBuf, args: Vec<String>) {
    if args.len() != 3 {
        panic!("Expected 3 arguments, but got {}", args.len());
    }

    let exe_path = path.join(format!("Simba-{}.exe", args[1]));
    let script_path = path.join("Scripts").join(args[0].clone() + ".simba");

    if !exe_path.exists() {
        let zip_path = path.join("Win64.zip");
        if zip_path.exists() {
            remove_file(&zip_path).expect("Failed to delete Win64.zip");
        }

        let url = "https://raw.githubusercontent.com/Villavu/Simba-Build-Archive/refs/heads/main/README.md";
        let res = reqwest::get(url).await.expect("Failed to fetch README.md");
        let body = res.text().await.expect("Failed to read response text");

        let search_token = format!("[{}]", args[1]);

        let line = body
            .lines()
            .find(|line| line.contains(&search_token))
            .expect("Branch not found in README.md");

        let mut win64_url = None;

        for part in line.split('[') {
            if part.starts_with("Win64](") {
                if let Some(start) = part.find("](") {
                    if let Some(end) = part[start + 2..].find(')') {
                        win64_url = Some(&part[start + 2..start + 2 + end]);
                        break;
                    }
                }
            }
        }

        let win64_url = win64_url.expect("No Win64 link found");
        let full_url = format!(
            "https://github.com/Villavu/Simba-Build-Archive/blob/main{}",
            win64_url
        );

        download_and_unzip(&full_url, &exe_path)
            .await
            .expect("Failed to download or unzip Win64.zip");
    }
}

#[tauri::command]
fn save_blob(file_path: String, data: Vec<u8>) -> Result<(), String> {
    let mut file = File::create(&file_path).map_err(|e| e.to_string())?;
    file.write_all(&data).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn run_executable(
    paths: State<'_, Mutex<ExecutablePaths>>,
    exe: String,
    args: Vec<String>,
) -> Result<String, String> {
    let args_clone = args.clone();

    let path = {
        let paths = paths.lock().unwrap();
        match exe.as_str() {
            "simba" => paths.simba.clone(),
            "runelite" => paths.runelite.clone(),
            "osclient" => paths.osclient.clone(),
            _ => paths.simba.clone(),
        }
    };

    if exe == "simba" {
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
        .plugin(tauri_plugin_fs::init())
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

            app.manage(Mutex::new(ExecutablePaths {
                simba: paths
                    .get("simba")
                    .and_then(|p: &serde_json::Value| p.as_str().map(PathBuf::from))
                    .unwrap_or_else(|| {
                        let path = app
                            .path()
                            .app_local_data_dir()
                            .expect("App Local Data Dir doesn't exist on this system")
                            .join("Simba");

                        let _ = ensure_simba_directories(&path);
                        let _ = sync_plugins_repo(&path.join("Plugins"));
                        path
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
            start_server,
            save_blob
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
