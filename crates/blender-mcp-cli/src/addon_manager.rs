use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderInstallation {
    pub version: String,
    pub path: PathBuf,
    pub addon_dir: PathBuf,
    pub is_addon_installed: bool,
}

pub fn detect_blender_installations() -> Vec<BlenderInstallation> {
    let mut installs = Vec::new();
    // HOME-relative locations apply on macOS and Linux; Windows uses %APPDATA% below.
    #[cfg(not(target_os = "windows"))]
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    #[cfg(not(target_os = "windows"))]
    let home_path = std::path::Path::new(&home);

    #[cfg(target_os = "macos")]
    {
        let app_support = home_path.join("Library/Application Support/Blender");
        if app_support.exists() {
            if let Ok(entries) = std::fs::read_dir(&app_support) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        // Version folder like 4.0, 4.1, 4.2, 4.3, 5.0
                        if name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                            let addon_dir = path.join("scripts/addons/blender_mcp");
                            let is_installed = addon_dir.join("__init__.py").exists();
                            installs.push(BlenderInstallation {
                                version: name,
                                path: PathBuf::from("/Applications/Blender.app"),
                                addon_dir,
                                is_addon_installed: is_installed,
                            });
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let config_blender = home_path.join(".config/blender");
        if config_blender.exists() {
            if let Ok(entries) = std::fs::read_dir(&config_blender) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                            let addon_dir = path.join("scripts/addons/blender_mcp");
                            let is_installed = addon_dir.join("__init__.py").exists();
                            installs.push(BlenderInstallation {
                                version: name,
                                path: PathBuf::from("/usr/bin/blender"),
                                addon_dir,
                                is_addon_installed: is_installed,
                            });
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(app_data) = std::env::var("APPDATA") {
            let blender_app_data = PathBuf::from(app_data).join("Blender Foundation\\Blender");
            if blender_app_data.exists() {
                if let Ok(entries) = std::fs::read_dir(&blender_app_data) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            if name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                                let addon_dir = path.join("scripts\\addons\\blender_mcp");
                                let is_installed = addon_dir.join("__init__.py").exists();
                                installs.push(BlenderInstallation {
                                    version: name,
                                    path: PathBuf::from(
                                        "C:\\Program Files\\Blender Foundation\\Blender\\blender.exe",
                                    ),
                                    addon_dir,
                                    is_addon_installed: is_installed,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    installs.sort_by(|a, b| b.version.cmp(&a.version));
    installs
}

pub fn install_addon(target_version: Option<&str>) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut installs = detect_blender_installations();
    // A Blender that has never been opened has no config folder yet; use the installed app's version.
    if let Some(version) = blender_binary().as_deref().and_then(binary_version) {
        if !installs.iter().any(|i| i.version == version) {
            if let Some(root) = config_root() {
                installs.push(BlenderInstallation {
                    addon_dir: root
                        .join(&version)
                        .join("scripts")
                        .join("addons")
                        .join("blender_mcp"),
                    path: blender_binary().unwrap_or_default(),
                    version,
                    is_addon_installed: false,
                });
            }
        }
    }
    let mut installed_paths = Vec::new();

    let addon_src = include_str!("../../../addon/blender_mcp/__init__.py");

    for inst in installs {
        if let Some(v) = target_version {
            if inst.version != v {
                continue;
            }
        }

        std::fs::create_dir_all(&inst.addon_dir)?;
        let init_py = inst.addon_dir.join("__init__.py");
        std::fs::write(&init_py, addon_src)?;
        installed_paths.push(init_py);
    }

    Ok(installed_paths)
}

/// Where Blender keeps per-version user folders (`<root>/<major.minor>/scripts/addons`).
fn config_root() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    return std::env::var_os("HOME")
        .map(|h| PathBuf::from(h).join("Library/Application Support/Blender"));
    #[cfg(target_os = "linux")]
    return std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config/blender"));
    #[cfg(target_os = "windows")]
    return std::env::var_os("APPDATA")
        .map(|a| PathBuf::from(a).join("Blender Foundation").join("Blender"));
    #[allow(unreachable_code)]
    None
}

/// The Blender executable on this machine, if one is installed in the usual place.
pub fn blender_binary() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    #[cfg(target_os = "macos")]
    {
        candidates.push(PathBuf::from(
            "/Applications/Blender.app/Contents/MacOS/Blender",
        ));
        if let Some(home) = std::env::var_os("HOME") {
            candidates
                .push(PathBuf::from(home).join("Applications/Blender.app/Contents/MacOS/Blender"));
        }
    }
    #[cfg(target_os = "windows")]
    {
        let base = PathBuf::from("C:\\Program Files\\Blender Foundation");
        if let Ok(entries) = std::fs::read_dir(&base) {
            let mut dirs: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path().join("blender.exe"))
                .collect();
            dirs.sort();
            candidates.extend(dirs.into_iter().rev());
        }
    }
    if let Some(path) = std::env::var_os("PATH") {
        let name = if cfg!(windows) {
            "blender.exe"
        } else {
            "blender"
        };
        candidates.extend(std::env::split_paths(&path).map(|d| d.join(name)));
    }
    candidates.into_iter().find(|p| p.is_file())
}

/// `major.minor` of a Blender executable, from `blender --version` ("Blender 5.2.2").
pub fn binary_version(blender: &Path) -> Option<String> {
    let out = Command::new(blender)
        .arg("--version")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    parse_version(&String::from_utf8_lossy(&out.stdout))
}

fn parse_version(text: &str) -> Option<String> {
    let line = text.lines().find(|l| l.starts_with("Blender "))?;
    let mut parts = line
        .trim_start_matches("Blender ")
        .split(|c: char| c == '.' || c.is_whitespace());
    Some(format!("{}.{}", parts.next()?, parts.next()?))
}

/// Whether this Blender is open. It saves its preferences on quit, which would undo an enable made meanwhile.
pub fn blender_running(blender: &Path) -> bool {
    if cfg!(windows) {
        return Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq blender.exe", "/NH"])
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .to_lowercase()
                    .contains("blender.exe")
            })
            .unwrap_or(false);
    }
    let me = blender.display().to_string();
    Command::new("ps")
        .args(["-axo", "command="])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .any(|l| l.starts_with(&me))
        })
        .unwrap_or(false)
}

/// Turn the add-on on in this Blender's saved preferences, so it loads (and starts listening) every
/// time Blender opens. Runs Blender without a window; the add-on itself is not started here.
pub fn enable_addon(blender: &Path) -> Result<(), String> {
    const SCRIPT: &str = "import bpy\n\
prefs = bpy.context.preferences\n\
if 'blender_mcp' not in prefs.addons:\n    prefs.addons.new().module = 'blender_mcp'\n\
bpy.ops.wm.save_userpref()\n\
print('BLENDER_MCP_ENABLED')\n";
    let out = Command::new(blender)
        .args([
            "--background",
            "--python-exit-code",
            "1",
            "--python-expr",
            SCRIPT,
        ])
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("could not run {}: {e}", blender.display()))?;
    if out.status.success() && String::from_utf8_lossy(&out.stdout).contains("BLENDER_MCP_ENABLED")
    {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&out.stderr);
        Err(err
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("Blender did not confirm")
            .to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_from_blender_output() {
        let out = "Blender 5.2.2\n\tbuild date: 2026-10-01\n";
        assert_eq!(parse_version(out).as_deref(), Some("5.2"));
        assert_eq!(parse_version("Blender 4.0.0 Alpha").as_deref(), Some("4.0"));
        assert_eq!(parse_version("something else"), None);
    }
}
