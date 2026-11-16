//! Lina Desk "Send logs".
//!
//! Collects this device's Lina Desk logs into one file on the Desktop, so the
//! person can hand it to their helper. Nothing is uploaded from here.
//!
//! On an installed Windows client the part that talks to the network runs as
//! a background service whose log folder a normal user cannot read. That
//! process hands its recent log lines over the local channel the app already
//! uses for the ID (`tail_text`, served in `src/ipc.rs` as "lina-logs").

use hbb_common::{config::Config, log};
use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

/// Newest files first, at most this many per folder tree.
const MAX_FILES: usize = 40;
/// Only the end of a very large file is kept.
const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;
/// What the background service sends back.
const SERVICE_TAIL_BYTES: usize = 256 * 1024;

fn log_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut dirs = vec![dir.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            // No symlinks: only plain files and folders under the log folder.
            let Ok(meta) = fs::symlink_metadata(&path) else {
                continue;
            };
            if meta.is_dir() {
                dirs.push(path);
            } else if meta.is_file() {
                let modified = meta.modified().ok();
                files.push((modified, path));
            }
        }
    }
    files.sort_by(|a, b| b.0.cmp(&a.0));
    files.into_iter().take(MAX_FILES).map(|(_, p)| p).collect()
}

fn read_tail(path: &Path, max: u64) -> Option<Vec<u8>> {
    let mut file = fs::File::open(path).ok()?;
    let len = file.metadata().ok()?.len();
    if len > max {
        file.seek(SeekFrom::Start(len - max)).ok()?;
    }
    let mut buf = Vec::new();
    file.take(max).read_to_end(&mut buf).ok()?;
    Some(buf)
}

/// The recent log lines of this process' own log folder, newest file last.
pub fn tail_text() -> String {
    let dir = Config::log_path();
    let mut parts: Vec<String> = Vec::new();
    let mut left = SERVICE_TAIL_BYTES;
    for path in log_files(&dir) {
        if left == 0 {
            break;
        }
        let Some(bytes) = read_tail(&path, left as u64) else {
            continue;
        };
        left = left.saturating_sub(bytes.len());
        let name = path
            .strip_prefix(&dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        parts.push(format!(
            "===== {} =====\n{}",
            name,
            String::from_utf8_lossy(&bytes)
        ));
    }
    parts.reverse();
    parts.join("\n")
}

fn about() -> String {
    #[cfg(windows)]
    let installed = crate::platform::is_installed();
    #[cfg(not(windows))]
    let installed = false;
    format!(
        "{} {}\nbuilt {}\nsystem {} {}\nsetup finished: {}\ncollected {}\n",
        crate::lina_policy::DISPLAY_NAME,
        crate::VERSION,
        crate::BUILD_DATE,
        std::env::consts::OS,
        std::env::consts::ARCH,
        installed,
        hbb_common::chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
    )
}

fn stage(staging: &Path) -> std::io::Result<()> {
    fs::create_dir_all(staging)?;
    fs::write(staging.join("about.txt"), about())?;
    let dir = Config::log_path();
    for path in log_files(&dir) {
        let Some(bytes) = read_tail(&path, MAX_FILE_BYTES) else {
            continue;
        };
        let relative = path.strip_prefix(&dir).unwrap_or(&path);
        let target = staging.join("app").join(relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(target, bytes)?;
    }
    #[cfg(windows)]
    if crate::platform::is_installed() {
        match crate::ipc::get_lina_service_logs() {
            Some(text) if !text.is_empty() => fs::write(staging.join("service.log"), text)?,
            _ => log::warn!("the background service sent no log lines"),
        }
    }
    Ok(())
}

fn desktop_dir() -> PathBuf {
    hbb_common::directories_next::UserDirs::new()
        .and_then(|dirs| dirs.desktop_dir().map(|p| p.to_path_buf()))
        .filter(|p| p.is_dir())
        .unwrap_or_else(std::env::temp_dir)
}

#[cfg(windows)]
fn zip(staging: &Path, target: &Path) -> bool {
    use std::os::windows::process::CommandExt;
    // tar.exe ships with Windows 10 and later and writes zip files with `-a`.
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_owned());
    let tar = Path::new(&root).join("System32").join("tar.exe");
    std::process::Command::new(tar)
        .arg("-a")
        .arg("-c")
        .arg("-f")
        .arg(target)
        .arg("-C")
        .arg(staging)
        .arg(".")
        .creation_flags(winapi::um::winbase::CREATE_NO_WINDOW)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
        && target.is_file()
}

#[cfg(not(windows))]
fn zip(staging: &Path, target: &Path) -> bool {
    std::process::Command::new("zip")
        .arg("-r")
        .arg("-q")
        .arg(target)
        .arg(".")
        .current_dir(staging)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
        && target.is_file()
}

fn reveal(path: &Path) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // Explorer wants the path quoted inside the switch, not the whole
        // argument quoted, so pass it through as written.
        let _ = std::process::Command::new("explorer.exe")
            .raw_arg(format!("/select,\"{}\"", path.display()))
            .spawn();
    }
    #[cfg(not(windows))]
    let _ = path;
}

/// Collect the logs. Returns the path of the file (or folder) on the Desktop,
/// or an empty string when nothing could be collected.
pub fn collect() -> String {
    let stamp = hbb_common::chrono::Local::now().format("%Y%m%d-%H%M%S");
    let name = format!("{}-logs-{}", crate::lina_policy::APP_NAME, stamp);
    let staging = std::env::temp_dir().join(&name);
    if let Err(err) = stage(&staging) {
        log::error!("could not collect the logs: {err}");
        let _ = fs::remove_dir_all(&staging);
        return String::new();
    }
    let target = desktop_dir().join(format!("{name}.zip"));
    let result = if zip(&staging, &target) {
        let _ = fs::remove_dir_all(&staging);
        target
    } else {
        // No zip tool on this system: hand over the plain folder instead.
        log::warn!("could not zip the logs, leaving the folder");
        staging
    };
    reveal(&result);
    result.to_string_lossy().to_string()
}
