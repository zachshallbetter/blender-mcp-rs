use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderInstallation {
    pub version: String,
    pub path: PathBuf,
    pub addon_dir: PathBuf,
    pub is_addon_installed: bool,
}

pub fn detect_blender_installations() -> Vec<BlenderInstallation> {
    let mut installs = Vec::new();
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let home_path = Path::new(&home);

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
                        if name.chars().next().map_or(false, |c| c.is_ascii_digit()) {
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

    installs.sort_by(|a, b| b.version.cmp(&a.version));
    installs
}

pub fn install_addon(target_version: Option<&str>) -> Result<Vec<PathBuf>, std::io::Error> {
    let installs = detect_blender_installations();
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
