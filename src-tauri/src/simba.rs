use std::{
    fs::{self, create_dir_all, remove_dir_all, remove_file, write, File},
    io::{self, BufRead, BufReader, Cursor},
    path::{Path, PathBuf},
};

use serde::Deserialize;
use tauri::{
    http::{HeaderMap, HeaderValue},
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

    if archive.len() != 1 {
        return Err(format!("Expected 1 file in ZIP, found {}", archive.len()).into());
    }

    let mut file = archive.by_index(0)?;
    if file.name().ends_with('/') {
        return Err("Unexpected directory in zip".into());
    }

    // Ensure parent directory exists
    if let Some(parent) = dest.parent() {
        create_dir_all(parent)?;
    }

    // Write the file using `dest` as the output path
    let mut out_file = File::create(dest)?;
    std::io::copy(&mut file, &mut out_file)?;

    Ok(())
}

async fn download_and_unzip_directory(
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
        println!("Downloading WaspLib {} from {}", src, url);

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

    println!("WaspLib {}.zip extracted to {:?}", src, path);

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
    // Build the headers
    let mut headers = HeaderMap::new();
    headers.insert("apikey", HeaderValue::from_static(SUPABASE_ANON_KEY));
    headers.insert("Accept", HeaderValue::from_static("application/json"));
    headers.insert("Accept-Profile", HeaderValue::from_static("scripts"));

    // Build the URL with query parameters
    let url =
        SUPABASE_URL.to_string() + "rest/v1/plugins?select=version&order=created_at.desc&limit=1";

    let client = Client::new();
    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await?
        .error_for_status()?; // ensure HTTP 2xx

    // Parse JSON response
    let plugins: Vec<Plugin> = response.json().await?;

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
        download_and_unzip_directory(parent_dir.to_path_buf(), "wasp-plugins", "plugins", &latest)
            .await;

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

pub async fn run_simba(path: PathBuf, args: Vec<String>) {
    println!("Attempt to run Simba from: {:?}", path);

    if args.len() != 6 {
        panic!("Expected 6 arguments, but got {}", args.len());
    }

    const URL: &'static str =
        "https://raw.githubusercontent.com/Villavu/Simba-Build-Archive/refs/heads/main/README.md";

    let mut body: Option<String> = None;

    let commit = if args[1] == "latest" {
        println!("Finding latest Simba available");

        let res = reqwest::get(URL).await.expect("Failed to fetch README.md");
        let text = res.text().await.expect("Failed to read response text");
        body = Some(text);

        let line = body
            .as_ref()
            .unwrap()
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
        args[1].to_string()
    };

    let exe_path = path.join(format!("Simba-{}.exe", commit));

    if !exe_path.exists() {
        println!("Downloading Simba-{}.exe", commit);
        let zip_path = path.join("Win64.zip");
        if zip_path.exists() {
            fs::remove_file(&zip_path).expect("Failed to delete Win64.zip");
        }

        // only fetch if we don’t already have a body
        if body.is_none() {
            let res = reqwest::get(URL).await.expect("Failed to fetch README.md");
            body = Some(res.text().await.expect("Failed to read response text"));
        }

        let search_token = format!("[{}]", commit);
        let cleanbody = body.as_ref().unwrap().replace("<br>", " ");

        let line = cleanbody
            .lines()
            .find(|line| line.contains(&search_token))
            .expect("Commit not found in README.md");

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

        download_and_unzip_file(&full_url, &exe_path)
            .await
            .expect("Failed to download or unzip Win64.zip");
    }

    if args[2] != "none" {
        let _ = download_and_unzip_directory(path.join("Includes"), "WaspLib", "wasplib", &args[2])
            .await;
    }

    let script_file: String = path
        .join("Scripts")
        .join(args[0].clone())
        .to_string_lossy()
        .to_string();

    let mut cmd = std::process::Command::new(exe_path);
    cmd.arg(script_file)
        .env("SCRIPT_ID", &args[3])
        .env("SCRIPT_REVISION", &args[4])
        .env("WASP_REFRESH_TOKEN", &args[5]);

    if args[1] != "latest" {
        cmd.env("SCRIPT_SIMBA_VERSION", &args[1]);
    }

    if (args[2] != "latest") && (args[2] != "none") {
        cmd.env("SCRIPT_WASPLIB_VERSION", &args[2]);
    }

    let _ = cmd.spawn().map_err(|err| err.to_string());
}
