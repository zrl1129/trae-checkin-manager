use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use directories::ProjectDirs;
use serde::{de::DeserializeOwned, Serialize};

pub fn get_data_dir() -> PathBuf {
    let proj = ProjectDirs::from("com", "trae", "checkin-manager")
        .expect("Failed to get project directories");
    let dir = proj.data_dir().to_path_buf();
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    dir
}

pub fn get_data_file(filename: &str) -> PathBuf {
    get_data_dir().join(filename)
}

pub fn read_json<T: DeserializeOwned + Default>(filename: &str) -> Result<T> {
    let path = get_data_file(filename);
    if !path.exists() {
        return Ok(T::default());
    }
    let content = fs::read_to_string(&path)?;
    if content.trim().is_empty() {
        return Ok(T::default());
    }
    let data: T = serde_json::from_str(&content)?;
    Ok(data)
}

pub fn write_json<T: Serialize>(filename: &str, data: &T) -> Result<()> {
    let path = get_data_file(filename);
    let content = serde_json::to_string_pretty(data)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, &content)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}
