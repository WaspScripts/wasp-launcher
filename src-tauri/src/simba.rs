use std::{
    fs::{create_dir_all, remove_dir_all, remove_file, write, File},
    io::{self, BufRead, BufReader, Cursor},
    path::{Path, PathBuf},
    process::Stdio,
    thread,
};

use serde::Deserialize;
use tauri::{
    http::{HeaderMap, HeaderValue},
    ipc::Channel,
    Error,
};
use tauri_plugin_http::reqwest::{self, Client};
use zip::ZipArchive;

const SUPABASE_URL: &str = "https://db.waspscripts.dev/";
const SUPABASE_ANON_KEY: &str = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJzdXBhYmFzZSIsImlhdCI6MTc1MTA0MTIwMCwiZXhwIjo0OTA2NzE0ODAwLCJyb2xlIjoiYW5vbiJ9.C_KW5x45BpIyOQrnZc7CKYKjHe0yxB4l-fTSC4z_kYY";

#[derive(Deserialize, Debug)]
struct Plugin {
    version: String,
}

async fn download_and_unzip_file(
    url: &str,
    dest: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = Client::new()
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

async fn download_and_unzip_dir(
    path: PathBuf,
    dest: &str,
    db_path: &str,
    src: &str,
) -> Result<(), Box<dyn std::error::Error>> {
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

        let response = Client::new()
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
    let file = File::open(&zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    create_dir_all(&final_path)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = final_path.join(file.name());

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

    let client = Client::new();
    let response = client
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

pub async fn sync_plugins_repo(plugins_path: &PathBuf) -> Result<(), Error> {
    let current = read_plugins_version(&plugins_path.join("version.simba"))?;
    println!("Current plugins version: {}", current);

    let latest = fetch_plugins_version()
        .await
        .expect("Failed to fetch latest plugin versions");
    println!("Latest plugins version: {}", latest);
    if current == latest {
        return Ok(());
    }

    let parent_dir = plugins_path.join("..");
    let _ =
        download_and_unzip_dir(parent_dir.to_path_buf(), "wasp-plugins", "plugins", &latest).await;

    Ok(())
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
    let res = reqwest::get(SIMBA_ARCHIVE_URL)
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

pub async fn run_simba(path: PathBuf, args: Vec<String>) {
    println!("Attempt to run Simba from: {:?}", path);

    if args.len() != 7 {
        panic!("Expected 6 arguments, but got {}", args.len());
    }

    let exe_path = ensure_simba_executable(&path, &args[1]).await;

    if args[2] != "none" {
        let _ = download_and_unzip_dir(path.join("Includes"), "WaspLib", "wasplib", &args[2]).await;
    }

    let script_file = path.join("Scripts").join(&args[0]);

    let mut cmd = std::process::Command::new(exe_path);
    cmd.arg("--open")
        .arg(script_file)
        .env("SCRIPT_ID", &args[3])
        .env("SCRIPT_REVISION", &args[4])
        .env("WASP_REFRESH_TOKEN", &args[5])
        .env("assets", &args[6]);

    if args[1] != "latest" {
        cmd.env("SCRIPT_SIMBA_VERSION", &args[1]);
    }

    if (args[2] != "latest") && (args[2] != "none") {
        cmd.env("SCRIPT_WASPLIB_VERSION", &args[2]);
    }

    let _ = cmd.spawn().map_err(|err| err.to_string());
}

pub async fn run_simba_script(
    path: PathBuf,
    target: isize,
    args: Vec<String>,
    channel: Channel<String>,
) -> Result<std::process::Child, String> {
    println!("Attempt to run Simba from: {:?}", path);

    if args.len() != 7 {
        return Err(format!("Expected 6 arguments, but got {}", args.len()));
    }

    let exe_path = ensure_simba_executable(&path, &args[1]).await;

    if args[2] != "none" {
        let _ = download_and_unzip_dir(path.join("Includes"), "WaspLib", "wasplib", &args[2]).await;
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
        .env("WASP_REFRESH_TOKEN", &args[5])
        .env("assets", &args[6]);

    if args[1] != "latest" {
        cmd.env("SCRIPT_SIMBA_VERSION", &args[1]);
    }

    if (args[2] != "latest") && (args[2] != "none") {
        cmd.env("SCRIPT_WASPLIB_VERSION", &args[2]);
    }

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
