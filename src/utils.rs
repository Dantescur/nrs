use crate::error::NrsError;
use dirs::home_dir;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

pub fn derive_registry_name(url: &str, custom_registries: &HashMap<String, String>) -> String {
    let host = url
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .split('/')
        .next()
        .unwrap_or("unknown")
        .replace(['.', ':'], "-");

    if !custom_registries.contains_key(&host) {
        return host;
    }

    let mut counter = 1;
    loop {
        let name = format!("{}-{}", host, counter);
        if !custom_registries.contains_key(&name) {
            return name;
        }
        counter += 1;
    }
}

pub fn get_config_path() -> Result<PathBuf, NrsError> {
    let home = home_dir().ok_or(NrsError::HomeDirNotFound)?;
    Ok(home.join(".nrsrc"))
}

pub fn get_npmrc_path() -> Result<PathBuf, NrsError> {
    let home = home_dir().ok_or(NrsError::HomeDirNotFound)?;
    Ok(home.join(".npmrc"))
}

pub fn get_npmrc_path_local(local: bool) -> Result<PathBuf, NrsError> {
    if !local {
        get_npmrc_path()
    } else {
        Ok(std::env::current_dir()?.join(".npmrc"))
    }
}

pub fn get_local_registry_url() -> Result<Option<String>, NrsError> {
    let npmrc_path = get_npmrc_path_local(true)?;
    if !npmrc_path.exists() {
        return Ok(None);
    }

    let file = File::open(&npmrc_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.trim().starts_with("registry=") {
            let url = line
                .trim()
                .strip_prefix("registry=")
                .unwrap_or("")
                .to_string();
            return Ok(Some(url));
        }
    }

    Ok(None)
}
