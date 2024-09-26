use std::{fs, path::PathBuf};

use anyhow::Context;

fn app_subdir(subdir: &str) -> anyhow::Result<PathBuf> {
    let path = dirs::data_local_dir()
        .expect("Could not get user data directory")
        .join("kodjin-cli")
        .join(subdir);
    fs::create_dir_all(&path).context("Could not create application directory")?;
    Ok(path)
}

pub fn downloads_dir() -> anyhow::Result<PathBuf> {
    app_subdir("downloads")
}

pub fn packages_dir() -> anyhow::Result<PathBuf> {
    app_subdir("packages")
}

pub fn logs_dir() -> anyhow::Result<PathBuf> {
    app_subdir("logs")
}
