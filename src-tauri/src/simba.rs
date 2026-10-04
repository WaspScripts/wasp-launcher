use std::{
    fs::{create_dir_all, remove_dir_all, remove_file, write, File},
    io::{self, BufRead, BufReader, Cursor},
    path::{Path, PathBuf},
    process::Stdio,
    sync::LazyLock,
    thread,
};

use serde::Deserialize;
use tauri::{
    http::{HeaderMap, HeaderValue},
    ipc::Channel,
    AppHandle, Error,
};
use tauri_plugin_http::reqwest::Client;
use zip::ZipArchive;

const SUPABASE_URL: &str = "https://db.waspscripts.dev/";
// Shared so every request reuses the same connection pool and TLS setup.
pub static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

const SUPABASE_ANON_KEY: &str = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJzdXBhYmFzZSIsImlhdCI6MTc1MTA0MTIwMCwiZXhwIjo0OTA2NzE0ODAwLCJyb2xlIjoiYW5vbiJ9.C_KW5x45BpIyOQrnZc7CKYKjHe0yxB4l-fTSC4z_kYY";

#[derive(Deserialize, Debug)]
struct Plugin {
    version: String,
}

async fn download_and_unzip_file(
    url: &str,
    dest: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = HTTP_CLIENT
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    let cursor = Cursor::new(response);
    let mut archive = ZipArchive::new(cursor)?;

    let files: Vec<String> = archive
        .file_names()
        .filter(|name| !name.ends_with('/'))
        .map(String::from)
        .collect();

    if files.len() != 1 {
        return Err(format!("Expected 1 file in ZIP, found {}", files.len()).into());
    }

    let mut file = archive.by_name(&files[0])?;

    // Ensure parent directory exists
    if let Some(parent) = dest.parent() {
        create_dir_all(parent)?;
    }

    // Write the file using `dest` as the output path
    let mut out_file = File::create(dest)?;
    std::io::copy(&mut file, &mut out_file)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o755))?;
    }

    Ok(())
}

type BoxError = Box<dyn std::error::Error + Send + Sync>;

// Extracts `zip_path` into `dest`, dropping the first `strip_components` folders of every entry.
fn extract_zip(zip_path: &Path, dest: &Path, strip_components: usize) -> Result<(), BoxError> {
    let mut archive = ZipArchive::new(File::open(zip_path)?)?;
    create_dir_all(dest)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let Some(name) = file.enclosed_name() else {
            continue;
        };

        let relative: PathBuf = name.components().skip(strip_components).collect();
        if relative.as_os_str().is_empty() {
            continue;
        }

        let out_path = dest.join(relative);
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

    Ok(())
}

async fn download_and_unzip_dir(
    path: PathBuf,
    dest: &str,
    db_path: &str,
    src: &str,
) -> Result<(), BoxError> {
    let final_path = path.join(dest);
    let zip_path = path.join(format!("{}.zip", src));

    if final_path.exists() {
        println!("Removing old {:?} directory", final_path);
        remove_dir_all(&final_path)?;
    }

    if src == "latest" && zip_path.exists() {
        let _ = remove_file(zip_path.clone());
    }

    if !zip_path.exists() {
        let url = format!("{}storage/v1/object/{}/{}.zip", SUPABASE_URL, db_path, src);
        println!("Downloading {} from {}", src, url);

        let response = HTTP_CLIENT
            .get(&url)
            .bearer_auth(SUPABASE_ANON_KEY)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        write(&zip_path, &response)?;
    }

    println!("Extracting {} to {:?}", zip_path.display(), final_path);
    extract_zip(&zip_path, &final_path, 0)?;

    println!("{}.zip extracted to {:?}", src, path);

    Ok(())
}

pub fn read_plugins_version(path: &Path) -> Result<String, Error> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Ok("Not installed".to_string());
        }
        Err(e) => return Err(e.into()),
    };
    let reader = BufReader::new(file);

    let mut year = None;
    let mut month = None;
    let mut day = None;
    let mut hash = None;

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.starts_with("WL_PLUGINS_VERSION_YEAR") {
            if let Some(val) = line.split('=').nth(1) {
                year = Some(
                    val.trim_end_matches(';')
                        .trim()
                        .parse::<u32>()
                        .expect("Failed to parse year!"),
                );
            }
        } else if line.starts_with("WL_PLUGINS_VERSION_MONTH") {
            if let Some(val) = line.split('=').nth(1) {
                month = Some(
                    val.trim_end_matches(';')
                        .trim()
                        .parse::<u32>()
                        .expect("Failed to parse month!"),
                );
            }
        } else if line.starts_with("WL_PLUGINS_VERSION_DAY") {
            if let Some(val) = line.split('=').nth(1) {
                day = Some(
                    val.trim_end_matches(';')
                        .trim()
                        .parse::<u32>()
                        .expect("Failed to parse day!"),
                );
            }
        } else if line.starts_with("WL_PLUGINS_VERSION_COMMIT_HASH") {
            if let Some(val) = line.split('=').nth(1) {
                let val = val.trim_end_matches(';').trim();
                hash = Some(val.trim_matches('\'').to_string());
            }
        }
    }

    let version = format!(
        "{}.{:02}.{:02}-{}",
        year.expect("Missing year"),
        month.expect("Missing month"),
        day.expect("Missing day"),
        hash.expect("Missing hash")
    );

    Ok(version)
}

async fn fetch_plugins_version() -> Result<String, Box<dyn std::error::Error>> {
    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_static(SUPABASE_ANON_KEY));
    headers.insert("Accept", HeaderValue::from_static("application/json"));
    headers.insert("Accept-Profile", HeaderValue::from_static("scripts"));

    let url =
        SUPABASE_URL.to_string() + "rest/v1/plugins?select=version&order=created_at.desc&limit=1";

    let response = HTTP_CLIENT
        .get(url)
        .headers(headers)
        .send()
        .await?
        .error_for_status()?; //ensure HTTP 2xx

    let body = response.text().await?;
    let plugins: Vec<Plugin> = serde_json::from_str(&body)?;

    if let Some(plugin) = plugins.first() {
        Ok(plugin.version.clone())
    } else {
        Err("No plugins found".into())
    }
}

pub async fn sync_plugins_repo(app: &AppHandle, plugins_path: &PathBuf) -> Result<(), Error> {
    let current = read_plugins_version(&plugins_path.join("version.simba"))?;
    println!("Current plugins version: {}", current);

    let latest = fetch_plugins_version()
        .await
        .expect("Failed to fetch latest plugin versions");
    println!("Latest plugins version: {}", latest);
    if current != latest {
        // Not `join("..")`: unlike Windows, Linux can't resolve ".." through the
        // wasp-plugins directory when it doesn't exist (yet, or after being removed).
        let parent_dir = plugins_path
            .parent()
            .expect("wasp-plugins path has no parent")
            .to_path_buf();
        if let Err(e) = download_and_unzip_dir(parent_dir, "wasp-plugins", "plugins", &latest).await
        {
            eprintln!("Failed to install wasp-plugins {}: {}", latest, e);
        }
    }

    // Ran on every sync so installing patchelf later fixes an already up to date install.
    clear_remote_input_execstack(app, plugins_path);

    Ok(())
}

// RemoteInput's library requests an executable stack, which newer glibc versions
// refuse to load, so the flag has to be cleared with patchelf.
#[cfg(target_os = "linux")]
fn clear_remote_input_execstack(app: &AppHandle, plugins_path: &Path) {
    use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

    let lib = plugins_path
        .join("libremoteinput")
        .join("libremoteinput64.so");
    if !lib.exists() {
        return;
    }

    let result = std::process::Command::new("patchelf")
        .arg("--clear-execstack")
        .arg(&lib)
        .output();

    let message = match result {
        Ok(output) if output.status.success() => return,
        Ok(output) => format!(
            "patchelf failed to patch {}:\n\n{}\n\nRemoteInput won't work until this is fixed. \
             Make sure your patchelf is version 0.18 or newer.",
            lib.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            "RemoteInput needs patchelf to work on Linux but it's not installed.\n\n\
             Install it with your package manager (e.g. \"sudo pacman -S patchelf\" or \
             \"sudo apt install patchelf\") and restart the launcher."
                .to_string()
        }
        Err(e) => format!("Failed to run patchelf: {}", e),
    };

    eprintln!("{}", message);
    app.dialog()
        .message(message)
        .title("patchelf required")
        .kind(MessageDialogKind::Warning)
        .show(|_| {});
}

#[cfg(not(target_os = "linux"))]
fn clear_remote_input_execstack(_app: &AppHandle, _plugins_path: &Path) {}

#[cfg(target_os = "linux")]
pub fn check_ptrace_scope(app: &AppHandle) -> Result<(), String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

    const PERSIST: &str =
        "echo 'kernel.yama.ptrace_scope = 0' | sudo tee /etc/sysctl.d/10-ptrace.conf";

    // Missing when Yama isn't enabled, in which case ptrace isn't restricted.
    let Ok(scope) = std::fs::read_to_string("/proc/sys/kernel/yama/ptrace_scope") else {
        return Ok(());
    };

    // Scope 3 is locked until a reboot, so it can only be persisted.
    let (command, steps) = match scope.trim() {
        "0" => return Ok(()),
        "3" => (
            PERSIST.to_string(),
            "Run this in a terminal and then restart your computer:",
        ),
        _ => (
            format!("{} && sudo sysctl -w kernel.yama.ptrace_scope=0", PERSIST),
            "Run this in a terminal to allow it:",
        ),
    };

    let message = format!(
        "RemoteInput needs ptrace to attach to the client but it's restricted on this system \
         (kernel.yama.ptrace_scope = {}).\n\n{}\n\n{}\n\n\
         Note: this is a system-wide setting, not just for Simba. \n\
         It lets any program running as your user inspect and modify your other programs \
         (e.g. read your browser's memory).\n\
         Programs of other users and root stay protected. \n\
         It's undone by deleting /etc/sysctl.d/10-ptrace.conf and rebooting.",
        scope.trim(),
        steps,
        command
    );

    eprintln!("{}", message);
    let handle = app.clone();
    app.dialog()
        .message(message)
        .title("ptrace restricted")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Copy command".into(),
            "Close".into(),
        ))
        .show(move |copy| {
            if copy {
                let _ = handle.clipboard().write_text(command);
            }
        });

    Err(format!(
        "ptrace is restricted (kernel.yama.ptrace_scope = {}), Simba wasn't launched.",
        scope.trim()
    ))
}

#[cfg(not(target_os = "linux"))]
pub fn check_ptrace_scope(_app: &AppHandle) -> Result<(), String> {
    Ok(())
}

// Simba is a GTK2/X11 app, so on Wayland it runs through XWayland and ignores GDK_SCALE.
// The only thing it honours is Xft.dpi, which is passed to it through XENVIRONMENT.
#[cfg(target_os = "linux")]
fn apply_dpi_scale(cmd: &mut std::process::Command, setting: f64) {
    let Some(scale) = resolve_dpi_scale(setting) else {
        return;
    };

    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let file = dir.join(format!("simba-xresources-{}", scale));
    let dpi = (96.0 * scale).round() as u32;

    if let Err(e) = write(&file, format!("Xft.dpi: {}\n", dpi)) {
        eprintln!("Failed to write {:?}: {}", file, e);
        return;
    }

    println!("Launching Simba at {}x scale ({} dpi)", scale, dpi);
    cmd.env("XENVIRONMENT", file);
}

#[cfg(not(target_os = "linux"))]
fn apply_dpi_scale(_cmd: &mut std::process::Command, _setting: f64) {}

// SIMBA_SCALE env var > launcher setting > auto detection. A setting of 0 means auto.
#[cfg(target_os = "linux")]
fn resolve_dpi_scale(setting: f64) -> Option<f64> {
    let env_scale = std::env::var("SIMBA_SCALE")
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .filter(|s| *s > 0.0);

    env_scale
        .or((setting > 0.0).then_some(setting))
        .or_else(hyprland_xwayland_scale)
}

// With xwayland:force_zero_scaling Hyprland draws X11 windows 1:1, so the app has to scale
// itself to the monitor's scale. Without it Hyprland upscales them and we must not scale twice.
#[cfg(target_os = "linux")]
fn hyprland_xwayland_scale() -> Option<f64> {
    std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE")?;

    let hyprctl = |args: &[&str]| -> Option<serde_json::Value> {
        let output = std::process::Command::new("hyprctl")
            .args(args)
            .arg("-j")
            .output()
            .ok()?;
        serde_json::from_slice(&output.stdout).ok()
    };

    let option = hyprctl(&["getoption", "xwayland:force_zero_scaling"])?;
    let zero_scaling = option["bool"]
        .as_bool()
        .or_else(|| option["int"].as_i64().map(|i| i != 0))?;
    if !zero_scaling {
        return None;
    }

    hyprctl(&["monitors"])?
        .as_array()?
        .iter()
        .find(|m| m["focused"].as_bool() == Some(true))?["scale"]
        .as_f64()
}

pub fn ensure_simba_directories(path: &PathBuf) -> std::io::Result<()> {
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

const SIMBA_ARCHIVE_URL: &str =
    "https://raw.githubusercontent.com/Villavu/Simba-Build-Archive/refs/heads/main/README.md";

async fn fetch_simba_archive() -> String {
    let res = HTTP_CLIENT
        .get(SIMBA_ARCHIVE_URL)
        .send()
        .await
        .expect("Failed to fetch README.md");
    res.text().await.expect("Failed to read response text")
}

async fn ensure_simba_executable(path: &Path, version: &str) -> PathBuf {
    let commit = if version == "latest" {
        println!("Finding latest Simba available");

        let body = fetch_simba_archive().await;
        let line = body
            .lines()
            .find(|l| l.contains("| simba2000 |"))
            .expect("Branch not found in README.md");

        let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        let commit_col = parts.get(2).expect("No commit column found");

        commit_col
            .split(']')
            .next()
            .and_then(|s| s.strip_prefix('['))
            .expect("Failed to parse commit")
            .to_string()
    } else {
        version.to_string()
    };

    let exe_name = format!("Simba-{}{}", commit, std::env::consts::EXE_SUFFIX);
    let exe_path = path.join(&exe_name);

    if !exe_path.exists() {
        println!("Downloading {}", exe_name);
        let url = simba_download_url(&commit).await;
        download_and_unzip_file(&url, &exe_path)
            .await
            .expect(&format!("Failed to download or unzip {}", exe_name));
    }

    exe_path
}

#[cfg(target_os = "windows")]
async fn simba_download_url(commit: &str) -> String {
    format!(
        "{}storage/v1/object/simba/{}/win64.zip",
        SUPABASE_URL, commit
    )
}

#[cfg(target_os = "linux")]
async fn simba_download_url(commit: &str) -> String {
    // Linux builds aren't mirrored on supabase, so get them from the build archive itself.
    let file = if cfg!(target_arch = "aarch64") {
        "Simba_linux_aarch64.zip"
    } else {
        "Simba_linux_x86_64.zip"
    };

    let body = fetch_simba_archive().await;
    let line = body
        .lines()
        .find(|l| l.contains(&format!("[{}]", commit)))
        .expect(&format!("Simba {} not found in README.md", commit));

    line.split(|c| c == '(' || c == ')')
        .find(|s| s.starts_with("https://") && s.ends_with(file))
        .expect(&format!("No Linux build found for Simba {}", commit))
        .to_string()
}

const CACHE_READER_PATH: &str = "utils/cache-reader";

#[derive(Deserialize)]
struct GitHubContent {
    sha: String,
}

// WaspLib zips are made with `git archive`, which leaves submodules out, so the
// cache-reader commit pinned by this WaspLib version is downloaded separately.
async fn download_cache_reader(includes: PathBuf, version: String) -> Result<PathBuf, BoxError> {
    let zip_path = includes.join(format!("cache-reader-{}.zip", version));
    if version != "latest" && zip_path.exists() {
        return Ok(zip_path);
    }

    // Every WaspLib version is a tag, "latest" is the head of the release branch.
    let git_ref = if version == "latest" {
        "release"
    } else {
        &version
    };
    let url = format!(
        "https://api.github.com/repos/WaspScripts/WaspLib/contents/{}?ref={}",
        CACHE_READER_PATH, git_ref
    );

    let client = &*HTTP_CLIENT;
    let body = client
        .get(&url)
        .header("User-Agent", "wasp-launcher")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let submodule: GitHubContent = serde_json::from_str(&body)?;

    let url = format!(
        "https://github.com/WaspScripts/cache-reader/archive/{}.zip",
        submodule.sha
    );
    println!("Downloading cache-reader {} from {}", submodule.sha, url);

    let response = client
        .get(&url)
        .header("User-Agent", "wasp-launcher")
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    write(&zip_path, &response)?;
    Ok(zip_path)
}

async fn install_wasplib(path: PathBuf, version: String) {
    let includes = path.join("Includes");
    let cache_reader =
        tauri::async_runtime::spawn(download_cache_reader(includes.clone(), version.clone()));

    if let Err(e) = download_and_unzip_dir(includes.clone(), "WaspLib", "wasplib", &version).await {
        eprintln!("Failed to install WaspLib {}: {}", version, e);
        return;
    }

    let zip_path = match cache_reader.await {
        Ok(Ok(zip_path)) => zip_path,
        Ok(Err(e)) => return eprintln!("Failed to download cache-reader: {}", e),
        Err(e) => return eprintln!("Failed to download cache-reader: {}", e),
    };

    // GitHub archives wrap everything in a "cache-reader-<sha>" folder.
    let dest = includes.join("WaspLib").join(CACHE_READER_PATH);
    println!("Extracting {} to {:?}", zip_path.display(), dest);
    if let Err(e) = extract_zip(&zip_path, &dest, 1) {
        eprintln!("Failed to extract cache-reader: {}", e);
    }
}

pub async fn run_simba(path: PathBuf, args: Vec<String>, dpi_scale: f64) {
    println!("Attempt to run Simba from: {:?}", path);

    if args.len() != 6 {
        panic!("Expected 6 arguments, but got {}", args.len());
    }

    // WaspLib doesn't depend on the Simba executable, so both are installed at the same time.
    let wasplib = (args[2] != "none")
        .then(|| tauri::async_runtime::spawn(install_wasplib(path.clone(), args[2].clone())));

    let exe_path = ensure_simba_executable(&path, &args[1]).await;

    if let Some(wasplib) = wasplib {
        let _ = wasplib.await;
    }

    let script_file = path.join("Scripts").join(&args[0]);

    let mut cmd = std::process::Command::new(exe_path);
    cmd.arg("--open")
        .arg(script_file)
        .env("SCRIPT_ID", &args[3])
        .env("SCRIPT_REVISION", &args[4])
        .env("WASP_REFRESH_TOKEN", &args[5]);

    if args[1] != "latest" {
        cmd.env("SCRIPT_SIMBA_VERSION", &args[1]);
    }

    if (args[2] != "latest") && (args[2] != "none") {
        cmd.env("SCRIPT_WASPLIB_VERSION", &args[2]);
    }

    apply_dpi_scale(&mut cmd, dpi_scale);

    let _ = cmd.spawn().map_err(|err| err.to_string());
}

pub async fn run_simba_script(
    path: PathBuf,
    target: isize,
    args: Vec<String>,
    dpi_scale: f64,
    channel: Channel<String>,
) -> Result<std::process::Child, String> {
    println!("Attempt to run Simba from: {:?}", path);

    if args.len() != 6 {
        return Err(format!("Expected 6 arguments, but got {}", args.len()));
    }

    // WaspLib doesn't depend on the Simba executable, so both are installed at the same time.
    let wasplib = (args[2] != "none")
        .then(|| tauri::async_runtime::spawn(install_wasplib(path.clone(), args[2].clone())));

    let exe_path = ensure_simba_executable(&path, &args[1]).await;

    if let Some(wasplib) = wasplib {
        let _ = wasplib.await;
    }

    let script_file: String = path
        .join("Scripts")
        .join(args[0].clone())
        .to_string_lossy()
        .to_string();

    let trgt = format!("--target={}", target);
    let mut cmd = std::process::Command::new(exe_path);

    cmd.arg(trgt)
        .arg("--keep-formatting")
        .arg("--run")
        .arg(script_file)
        .env("SCRIPT_ID", &args[3])
        .env("SCRIPT_REVISION", &args[4])
        .env("WASP_REFRESH_TOKEN", &args[5]);

    if args[1] != "latest" {
        cmd.env("SCRIPT_SIMBA_VERSION", &args[1]);
    }

    if (args[2] != "latest") && (args[2] != "none") {
        cmd.env("SCRIPT_WASPLIB_VERSION", &args[2]);
    }

    apply_dpi_scale(&mut cmd, dpi_scale);

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| e.to_string())?;
    println!("Sending messages to channel: {}", channel.id());

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    let process_stdout = channel.clone();
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().flatten() {
            let _ = process_stdout.send(line);
        }
    });

    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            let _ = channel.send(format!("ERROR: {}", line));
        }
    });

    Ok(child)
}
