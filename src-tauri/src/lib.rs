use std::{
    collections::HashMap,
    env,
    fs::{create_dir_all, remove_dir_all, remove_file, write, File},
    io::{Cursor, Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self},
};

use git2::{Cred, Error, FetchOptions, RemoteCallbacks, Repository, StashFlags};
use serde_json::json;
use tauri::{Emitter, Manager, State};
use tauri_plugin_store::StoreExt;

use tauri_plugin_cli::CliExt;
use tauri_plugin_http::reqwest::{self, Client};
use tauri_plugin_updater::UpdaterExt;
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

fn sync_plugins_repo(plugins_path: &PathBuf) -> Result<(), Error> {
    let repo = match Repository::open(plugins_path) {
        Ok(mut repo) => {
            if !repo.index()?.is_empty() {
                let sig = repo.signature()?;
                let _ = repo.stash_save(
                    &sig,
                    "Auto stash before pull",
                    Some(StashFlags::INCLUDE_UNTRACKED),
                );
            }
            repo
        }
        Err(_) => {
            println!("Cloning WaspScripts/wasp-plugins repo...");
            Repository::clone(
                "https://git.waspscripts.dev/WaspScripts/wasp-plugins.git",
                plugins_path,
            )?
        }
    };

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_, _, _| Cred::default());

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    repo.find_remote("origin")?
        .fetch(&["main"], Some(&mut fetch_opts), None)?;

    let fetch_head = repo.reference_to_annotated_commit(&repo.find_reference("FETCH_HEAD")?)?;
    let local_branch = repo.find_branch("main", git2::BranchType::Local)?;
    let analysis = repo.merge_analysis(&[&fetch_head])?;

    if analysis.0.is_fast_forward() {
        let mut reference = local_branch.into_reference();
        reference.set_target(fetch_head.id(), "Fast-forward")?;
        repo.set_head(reference.name().unwrap())?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;
    }

    Ok(())
}

async fn ensure_wasplib_at_tag(path: PathBuf, tag: &str) -> Result<(), Box<dyn std::error::Error>> {
    let lib_path = path.join("WaspLib");
    let zip_path = path.join(format!("{}.zip", tag));

    // 1. Delete WaspLib folder if it exists
    if lib_path.exists() {
        println!("Removing old WaspLib folder...");
        remove_dir_all(&lib_path)?;
    }

    // 2. Check if zip exists, otherwise download it
    if !zip_path.exists() {
        let url = format!(
            "https://db.waspscripts.dev/storage/v1/object/wasplib/{}.zip",
            tag
        );
        println!("Downloading WaspLib {} from {}", tag, url);

        let response = Client::new()
            .get(&url).bearer_auth("eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJzdXBhYmFzZSIsImlhdCI6MTc1MTA0MTIwMCwiZXhwIjo0OTA2NzE0ODAwLCJyb2xlIjoiYW5vbiJ9.C_KW5x45BpIyOQrnZc7CKYKjHe0yxB4l-fTSC4z_kYY")
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        write(&zip_path, &response)?;
    }

    // 3. Unzip into WaspLib/
    println!("Extracting {} to {:?}", zip_path.display(), lib_path);
    let file = File::open(&zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    create_dir_all(&lib_path)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = lib_path.join(file.name());

        if file.is_dir() {
            create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                create_dir_all(parent)?;
            }
            let mut outfile = File::create(&out_path)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    println!("WaspLib {} ready at {:?}", tag, lib_path);

    Ok(())
}

fn ensure_simba_directories(path: &PathBuf) -> std::io::Result<()> {
    create_dir_all(path)?;

    let dirs = [
        "Configs",
        "Data",
        "Includes",
        "Plugins",
        "Screenshots",
        "Scripts",
    ];

    for dir in &dirs {
        create_dir_all(&path.join(dir))?;
    }

    Ok(())
}

async fn download_and_unzip(url: &str, dest: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
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
    if args.len() != 6 {
        panic!("Expected 3 arguments, but got {}", args.len());
    }

    let exe_path = path.join(format!("Simba-{}.exe", args[1]));

    if !exe_path.exists() {
        println!("Downloading Simba-{}.exe", args[1]);
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

    let _ = ensure_wasplib_at_tag(path.join("Includes"), &args[2]).await;

    let script_file: String = path
        .join("Scripts")
        .join(args[0].clone())
        .to_string_lossy()
        .to_string();

    let _ = Command::new(exe_path)
        .args([script_file])
        .env("SCRIPT_SIMBA_VERSION", &args[1])
        .env("SCRIPT_WASPLIB_VERSION", &args[2])
        .env("SCRIPT_ID", &args[3])
        .env("SCRIPT_REVISION", &args[4])
        .env("WASP_REFRESH_TOKEN", &args[5])
        .spawn()
        .map_err(|err| err.to_string());
}

fn send_html(content: &str) -> String {
    let html = format!(
        "<!DOCTYPE html>\n\
         <html>\n\
           <head>\n\
             <meta charset=\"UTF-8\">\n\
             <link rel=\"icon\" href=\"https://waspscripts.com/favicon.png\">\n\
             <meta name=\"viewport\" content=\"width=device-width\">\n\
             <title>WaspScripts</title>\n\
             <meta name=\"description\" content=\"WaspScripts Simba Login page\">\n\
             <style>\n\
               body {{\n\
                  background-color: #222324;\n\
                  color: white;\n\
                  display: flex;\n\
                  justify-content: center;\n\
                  height: 100vh;\n\
                  text-align: center;\n\
                  flex-direction: column;\n\
               }}\n\
             </style>\n\
           </head>\n\
          <body>\n\
            {content}\n\
          </body>\n\
        </html>"
    );

    let headers = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/html\r\n\
         Connection: close\r\n\
         Content-Length: {}\r\n\r\n",
        html.len()
    );

    format!("{headers}{html}")
}

#[tauri::command]
fn save_blob(app: tauri::AppHandle, path: String, data: Vec<u8>) -> Result<(), String> {
    let final_path = app
        .path()
        .app_local_data_dir()
        .expect("App Local Data Dir doesn't exist on this system")
        .join("Simba")
        .join("Scripts")
        .join(path);

    let mut file = File::create(final_path).map_err(|e| e.to_string())?;
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

#[tauri::command]
async fn sign_up(id: String) -> Result<String, String> {
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

fn handle_client(mut stream: TcpStream, app: tauri::AppHandle) -> bool {
    let response = send_html("<h2>Authentication Complete</h2>\r\n        <p>You may now close this window and return to the app.</p>");

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

async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    if let Some(update) = app.updater()?.check().await? {
        let mut downloaded = 0;

        // alternatively we could also call update.download() and update.install() separately
        update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    println!("Downloaded {downloaded} from {content_length:?}");
                },
                || {
                    println!("Download finished");
                },
            )
            .await?;

        println!("Update installed!");
        app.restart();
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            match app.cli().matches() {
                Ok(matches) => {
                    // args is a HashMap<String, ArgData>
                    if let Some(arg) = matches.args.get("debug") {
                        if arg.occurrences > 0 {
                            println!("Debug flag present!");
                            window.open_devtools();
                        }
                    }
                }
                Err(_) => {}
            }

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if cfg!(dev) {
                    update(handle).await.unwrap();
                } else {
                    println!("Update available!")
                }
            });

            let settings = app.store("settings.json")?;

            let app_paths = app.path();
            let local_data = app_paths
                .app_local_data_dir()
                .expect("Local Data Dir doesn't exist on this system");

            let program_files: PathBuf = env::var("PROGRAMFILES(X86)")
                .map(PathBuf::from)
                .unwrap_or_else(|_| local_data.clone());

            let paths: serde_json::Value = settings.get("paths").unwrap_or_else(|| {
                let empty = json!({});
                settings.set("paths", empty.clone());
                empty
            });

            let get_path = |key: &str, fallback: PathBuf| {
                paths
                    .get(key)
                    .and_then(|v| v.as_str())
                    .map(PathBuf::from)
                    .unwrap_or(fallback)
            };

            let simba_path = local_data.join("Simba");
            let _ = ensure_simba_directories(&simba_path);

            let plugins_path = simba_path.join("Plugins").join("wasp-plugins");
            tauri::async_runtime::spawn(async move {
                let _ = sync_plugins_repo(&plugins_path);
            });

            let runelite_default = app_paths
                .local_data_dir()
                .expect("Local Data Dir doesn't exist on this system")
                .join("RuneLite\\RuneLite.exe");

            let osclient_default = program_files
                .join("Jagex Launcher\\Games\\Old School RuneScape\\Client\\osclient.exe");

            app.manage(Mutex::new(ExecutablePaths {
                simba: get_path("simba", simba_path),
                runelite: get_path("runelite", runelite_default),
                osclient: get_path("osclient", osclient_default),
            }));

            let _ = window.set_background_color(Some([25, 25, 25].into()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            set_executable_path,
            get_executable_path,
            run_executable,
            start_server,
            sign_up,
            save_blob
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
