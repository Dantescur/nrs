use colored::*;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufRead, BufReader, Read},
    time::Instant,
};

use serde::{Deserialize, Serialize};

use crate::{
    error::NrsError,
    utils::{derive_registry_name, get_config_path, get_npmrc_path, get_npmrc_path_local},
};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub registries: HashMap<String, String>,
    pub custom_registries: HashMap<String, String>,
    pub registry_order: Vec<String>,
    pub current: Option<String>,
}

impl Config {
    pub fn new() -> Self {
        let mut registries = HashMap::new();
        let mut registry_order = Vec::new();

        let default_registries = [
            ("npm", "https://registry.npmjs.org/"),
            ("yarn", "https://registry.yarnpkg.com/"),
            ("taobao", "https://registry.npmmirror.com/"),
            ("tencent", "https://mirrors.cloud.tencent.com/npm/"),
            ("npmMirror", "https://skimdb.npmjs.com/registry/"),
            ("github", "https://npm.pkg.github.com/"),
        ];

        for (name, url) in default_registries {
            registries.insert(name.to_string(), url.to_string());
            registry_order.push(name.to_string());
        }

        Config {
            registries,
            custom_registries: HashMap::new(),
            registry_order,
            current: None,
        }
    }

    pub fn load() -> Result<Self, NrsError> {
        let config_path = get_config_path()?;
        let mut config = if config_path.exists() {
            let mut file = File::open(&config_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            serde_json::from_str(&contents)?
        } else {
            Config::new()
        };

        if config.registry_order.is_empty() {
            config.registry_order = config
                .registries
                .keys()
                .chain(config.custom_registries.keys())
                .cloned()
                .collect();
            config.registry_order.sort();
        }

        let npmrc_path = get_npmrc_path()?;
        if npmrc_path.exists() {
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
                    let mut found = false;
                    // Check default registries
                    for (name, reg_url) in &config.registries {
                        if reg_url == &url {
                            config.current = Some(name.clone());
                            found = true;
                            break;
                        }
                    }
                    // Check custom registries
                    if !found {
                        for (name, reg_url) in &config.custom_registries {
                            if reg_url == &url {
                                config.current = Some(name.clone());
                                found = true;
                                break;
                            }
                        }
                    }
                    // Add unknown registry to custom_registries
                    if !found && !url.is_empty() {
                        let name = derive_registry_name(&url, &config.custom_registries);
                        config.custom_registries.insert(name.clone(), url.clone());
                        if !config.registry_order.contains(&name) {
                            config.registry_order.push(name.clone());
                        }
                        config.current = Some(name);
                    }
                    break;
                }
            }
        }

        config.save()?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), NrsError> {
        let config_path = get_config_path()?;
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, contents)?;
        Ok(())
    }

    pub fn set_current(&mut self, name: &str, backup: bool, local: bool) -> Result<(), NrsError> {
        if !self.registries.contains_key(name) && !self.custom_registries.contains_key(name) {
            return Err(NrsError::RegistryNotFound(name.to_string()));
        }
        self.current = Some(name.to_string());
        self.save()?;
        self.update_npmrc(backup, local)?;
        Ok(())
    }

    pub fn add_registry(&mut self, name: &str, url: &str) -> Result<(), NrsError> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(NrsError::InvalidRegistryUrl(url.to_string()));
        }
        // Check if URL already exists
        for (existing_name, existing_url) in
            self.registries.iter().chain(self.custom_registries.iter())
        {
            if existing_url == url {
                if existing_name != name {
                    println!(
                        "{} Registry URL {} already exists as {}. Use that name or edit it.",
                        "Warning:".yellow(),
                        url,
                        existing_name
                    );
                }
                return Ok(());
            }
        }
        self.custom_registries
            .insert(name.to_string(), url.to_string());
        if !self.registry_order.contains(&name.to_string()) {
            self.registry_order.push(name.to_string());
        }
        self.save()?;
        Ok(())
    }

    pub fn edit_registry(&mut self, name: &str, new_url: &str) -> Result<(), NrsError> {
        if !self.registries.contains_key(name) && !self.custom_registries.contains_key(name) {
            return Err(NrsError::RegistryNotFound(name.to_string()));
        }
        if !new_url.starts_with("https://") && !new_url.starts_with("http://") {
            return Err(NrsError::InvalidRegistryUrl(new_url.to_string()));
        }
        if self.registries.contains_key(name) {
            self.registries
                .insert(name.to_string(), new_url.to_string());
        } else {
            self.custom_registries
                .insert(name.to_string(), new_url.to_string());
        }
        self.save()?;
        Ok(())
    }

    pub fn remove_registry(&mut self, name: &str) -> Result<(), NrsError> {
        let was_default = self.registries.remove(name).is_some();
        let was_custom = self.custom_registries.remove(name).is_some();
        if !was_default && !was_custom {
            return Err(NrsError::RegistryNotFound(name.to_string()));
        }
        self.registry_order.retain(|n| n != name);
        if self.current.as_deref() == Some(name) {
            self.current = None;
        }
        self.save()?;
        Ok(())
    }

    pub fn update_npmrc(&self, backup: bool, local: bool) -> Result<(), NrsError> {
        let npmrc_path = get_npmrc_path_local(local)?;
        let new_registry = if let Some(current) = &self.current {
            self.registries
                .get(current)
                .or_else(|| self.custom_registries.get(current))
                .ok_or_else(|| NrsError::RegistryNotFound(current.to_string()))?
        } else {
            return Ok(());
        };

        if backup && npmrc_path.exists() {
            let backup_path = npmrc_path.with_extension("npmrc.bak");
            fs::copy(&npmrc_path, &backup_path)?;
        }

        let mut lines = Vec::new();
        if npmrc_path.exists() {
            let file = File::open(&npmrc_path)?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                if !line.trim().starts_with("registry=") {
                    lines.push(line);
                }
            }
        }
        lines.push(format!("registry={}", new_registry));
        let content = lines.join("\n") + "\n";
        fs::write(&npmrc_path, content)?;
        Ok(())
    }

    pub fn test_registry(&self, name: &str) -> Result<(bool, Option<u128>), NrsError> {
        let url = self
            .registries
            .get(name)
            .or_else(|| self.custom_registries.get(name))
            .ok_or_else(|| NrsError::RegistryNotFound(name.to_string()))?;
        let client = reqwest::blocking::Client::new();
        let start = Instant::now();
        let response = client.head(url).send();
        let duration = start.elapsed().as_millis();
        match response {
            Ok(resp) if resp.status().is_success() => Ok((true, Some(duration))),
            _ => Ok((false, Some(duration))),
        }
    }
}
