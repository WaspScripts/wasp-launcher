mod commands;
mod server;
mod simba;

use std::{env, path::PathBuf, sync::Mutex};

use serde_json::json;
use tauri::Manager;
use tauri_plugin_store::StoreExt;

use tauri_plugin_cli::CliExt;
use tauri_plugin_updater::UpdaterExt;

#[derive(Default)]
struct ExecutablePaths {
    simba: PathBuf,
    devsimba: PathBuf,
    runelite: PathBuf,
    osclient: PathBuf,
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
        .plugin(tauri_plugin_http::init())
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
            if !tauri::is_dev() {
                tauri::async_runtime::spawn(async move {
                    update(handle).await.unwrap();
                });
            }

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
            let _ = simba::ensure_simba_directories(&simba_path);

            let plugins_path = simba_path.join("Plugins").join("wasp-plugins");
            tauri::async_runtime::spawn(async move {
                println!("Started plugins async thread!");
                let _ = simba::sync_plugins_repo(&plugins_path).await;
            });

            let runelite_default = app_paths
                .local_data_dir()
                .expect("Local Data Dir doesn't exist on this system")
                .join("RuneLite\\RuneLite.exe");

            let osclient_default = program_files
                .join("Jagex Launcher\\Games\\Old School RuneScape\\Client\\osclient.exe");

            app.manage(Mutex::new(ExecutablePaths {
                simba: simba_path.clone(),
                devsimba: get_path("devsimba", simba_path),
                runelite: get_path("runelite", runelite_default),
                osclient: get_path("osclient", osclient_default),
            }));

            let _ = window.set_background_color(Some([25, 25, 25].into()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::set_executable_path,
            commands::get_executable_path,
            commands::run_executable,
            commands::start_server,
            commands::sign_up,
            commands::save_blob,
            commands::delete_cache,
            commands::delete_assets,
            commands::delete_configs,
            commands::get_plugin_version,
            commands::reinstall_plugins
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
