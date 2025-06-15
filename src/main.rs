use clap::{CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum, ValueHint};
use clap_autocomplete::{add_subcommand, test_subcommand};
use colored::*;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read};
use std::path::PathBuf;
use std::time::Instant;
use thiserror::Error;

#[derive(Error, Debug)]
enum NrsError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Clap error: {0}")]
    ClapFailed(#[from] clap::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Registry not found: {0}")]
    RegistryNotFound(String),
    #[error("Home directory not found")]
    HomeDirNotFound,
    #[error("Invalid registry URL: {0}")]
    InvalidRegistryUrl(String),
}

#[derive(Serialize, Deserialize)]
struct Config {
    registries: HashMap<String, String>,
    registry_order: Vec<String>,
    current: Option<String>,
}

impl Config {
    fn new() -> Self {
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
            registry_order,
            current: None,
        }
    }

    fn load() -> Result<Self, NrsError> {
        let config_path = get_config_path()?;
        let mut config = if config_path.exists() {
            let mut file = File::open(&config_path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            serde_json::from_str(&contents)?
        } else {
            Config::new()
        };

        // Ensure registry_order includes all registries
        if config.registry_order.is_empty() {
            config.registry_order = config.registries.keys().cloned().collect();
            config.registry_order.sort();
        }

        // Sync current with .npmrc
        let npmrc_path = get_npmrc_path()?;
        if npmrc_path.exists() {
            let file = File::open(&npmrc_path)?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line?;
                if line.trim().starts_with("registry=") {
                    let url = line.trim().strip_prefix("registry=").unwrap_or("");
                    // Find the registry name matching the URL
                    for (name, reg_url) in &config.registries {
                        if reg_url == url {
                            config.current = Some(name.clone());
                            break;
                        }
                    }
                    break;
                }
            }
        }

        // Save updated config to persist current
        config.save()?;

        Ok(config)
    }

    fn save(&self) -> Result<(), NrsError> {
        let config_path = get_config_path()?;
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, contents)?;
        Ok(())
    }

    fn set_current(&mut self, name: &str, backup: bool) -> Result<(), NrsError> {
        if !self.registries.contains_key(name) {
            return Err(NrsError::RegistryNotFound(name.to_string()));
        }
        self.current = Some(name.to_string());
        self.save()?;
        self.update_npmrc(backup)?;
        Ok(())
    }

    fn add_registry(&mut self, name: &str, url: &str) -> Result<(), NrsError> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(NrsError::InvalidRegistryUrl(url.to_string()));
        }
        self.registries.insert(name.to_string(), url.to_string());
        if !self.registry_order.contains(&name.to_string()) {
            self.registry_order.push(name.to_string());
        }
        self.save()?;
        Ok(())
    }

    fn edit_registry(&mut self, name: &str, new_url: &str) -> Result<(), NrsError> {
        if !self.registries.contains_key(name) {
            return Err(NrsError::RegistryNotFound(name.to_string()));
        }
        if !new_url.starts_with("https://") && !new_url.starts_with("http://") {
            return Err(NrsError::InvalidRegistryUrl(new_url.to_string()));
        }
        self.registries
            .insert(name.to_string(), new_url.to_string());
        self.save()?;
        Ok(())
    }

    fn remove_registry(&mut self, name: &str) -> Result<(), NrsError> {
        if self.registries.remove(name).is_none() {
            return Err(NrsError::RegistryNotFound(name.to_string()));
        }
        self.registry_order.retain(|n| n != name);
        if self.current.as_deref() == Some(name) {
            self.current = None;
        }
        self.save()?;
        Ok(())
    }

    fn update_npmrc(&self, backup: bool) -> Result<(), NrsError> {
        let npmrc_path = get_npmrc_path()?;
        let new_registry = if let Some(current) = &self.current {
            self.registries
                .get(current)
                .ok_or_else(|| NrsError::RegistryNotFound(current.to_string()))?
        } else {
            return Ok(());
        };

        // Create backup if requested and .npmrc exists
        if backup && npmrc_path.exists() {
            let backup_path = npmrc_path.with_extension("npmrc.bak");
            fs::copy(&npmrc_path, &backup_path)?;
        }

        let mut lines = Vec::new();

        // Read existing .npmrc if it exists
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

        // Append the new registry line
        lines.push(format!("registry={}", new_registry));

        // Write the updated content back
        let content = lines.join("\n") + "\n"; // Ensure trailing newline
        fs::write(&npmrc_path, content)?;
        Ok(())
    }

    fn test_registry(&self, name: &str) -> Result<(bool, Option<u128>), NrsError> {
        let url = self
            .registries
            .get(name)
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

fn get_config_path() -> Result<PathBuf, NrsError> {
    let home = home_dir().ok_or(NrsError::HomeDirNotFound)?;
    Ok(home.join(".nrsrc"))
}

fn get_npmrc_path() -> Result<PathBuf, NrsError> {
    let home = home_dir().ok_or(NrsError::HomeDirNotFound)?;
    Ok(home.join(".npmrc"))
}

#[derive(Clone, Copy, ValueEnum)]
enum SortOrder {
    Name,
    Url,
    Default,
}

#[derive(Parser)]
#[command(name = "nrs", about = "Node.js registry switcher")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all registries
    Ls {
        #[arg(long, default_value = "name")]
        sort: SortOrder,
    },
    /// Use a specific registry
    Use {
        #[arg(value_hint = ValueHint::Other)]
        name: String,
        #[arg(long)]
        backup: bool,
    },
    /// Add a new registry
    Add {
        #[arg(value_hint = ValueHint::Other)]
        name: String,
        #[arg(value_hint = ValueHint::Url)]
        url: String,
    },
    /// Remove a registry
    Remove {
        #[arg(value_hint = ValueHint::Other)]
        name: String,
    },
    /// Edit a registry
    Edit {
        #[arg(value_hint = ValueHint::Other)]
        name: String,

        #[arg(value_hint = ValueHint::Url)]
        new_url: String,
    },
    /// Show current registry
    Current,
    /// Show current npmrc
    Show,
    /// Test registry availability
    Test {
        #[arg(default_value = "", value_hint = ValueHint::Other)]
        name: String,
    },
}

fn main() -> Result<(), NrsError> {
    let mut command = Cli::command();
    let command_clone = command.clone();

    command = add_subcommand(command);

    let matches = command.get_matches();

    if let Some(result) = test_subcommand(&matches, command_clone) {
        if let Err(err) = result {
            eprintln!("Error al instalar el autocompletado: {err}");
            std::process::exit(1);
        } else {
            std::process::exit(0);
        }
    }

    let cli = Cli::from_arg_matches(&matches)?;
    let mut config = Config::load()?;

    match cli.command {
        Commands::Ls { sort } => {
            let mut registries: Vec<(String, String)> = config
                .registries
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            match sort {
                SortOrder::Name => registries.sort_by(|a, b| a.0.cmp(&b.0)),
                SortOrder::Url => registries.sort_by(|a, b| a.1.cmp(&b.1)),
                SortOrder::Default => {
                    registries.sort_by(|a, b| {
                        let a_idx = config
                            .registry_order
                            .iter()
                            .position(|n| n == &a.0)
                            .unwrap_or(usize::MAX);
                        let b_idx = config
                            .registry_order
                            .iter()
                            .position(|n| n == &b.0)
                            .unwrap_or(usize::MAX);
                        a_idx.cmp(&b_idx)
                    });
                }
            }
            for (name, url) in registries {
                let marker = if config.current.as_deref() == Some(&name) {
                    format!("{}", "🟢".green())
                } else {
                    "".to_string()
                };
                let name_str = if config.current.as_deref() == Some(&name) {
                    format!("{}", name.bold().green())
                } else {
                    name.normal().white().to_string()
                };
                println!("{:2} {:15} {}", marker, name_str, url)
            }
        }
        Commands::Use { name, backup } => {
            config.set_current(&name, backup)?;
            println!("Switched to registry: {}", name);
        }
        Commands::Add { name, url } => {
            config.add_registry(&name, &url)?;
            println!("Added registry: {} ({})", name, url);
        }
        Commands::Remove { name } => {
            config.remove_registry(&name)?;
            println!("Removed registry: {}", name);
        }
        Commands::Current => {
            if let Some(current) = &config.current {
                let url = &config.registries[current];
                println!(
                    "{} {} ({})",
                    "Current registry:".bold(),
                    current.green().bold(),
                    url
                );
            } else {
                println!("{}", "No registry selected".yellow());
            }
        }
        Commands::Test { name } => {
            if name.is_empty() {
                // Test all registries
                let mut registries: Vec<_> = config.registries.keys().collect();
                registries.sort();
                for name in registries {
                    let (ok, duration) = config.test_registry(name)?;
                    let status = if ok {
                        format!("{}", "✅ OK".green())
                    } else {
                        format!("{}", "🔴 Failed".red())
                    };
                    let time = duration.map(|d| format!(" ({}ms)", d)).unwrap_or_default();
                    let current = if config.current.as_deref() == Some(name) {
                        format!("{}", "🟢".green()).to_string()
                    } else {
                        "".to_string()
                    };
                    println!(
                        "{:2} {:15} {} - {}{}",
                        current, name, status, config.registries[name], time
                    );
                }
            } else {
                // Test a single registry
                let (ok, duration) = config.test_registry(&name)?;
                let status = if ok { "OK" } else { "Failed" };
                let time = duration.map(|d| format!(" ({}ms)", d)).unwrap_or_default();
                println!(
                    "{}: {} - {}{}",
                    name, status, config.registries[&name], time
                );
            }
        }
        Commands::Edit { name, new_url } => {
            config.edit_registry(&name, &new_url)?;
        }
        Commands::Show => {
            let npmrc_path = get_npmrc_path()?;
            if npmrc_path.exists() {
                let content = fs::read_to_string(npmrc_path)?;
                println!("{}", content);
            } else {
                println!("{}", "No .npmrc file found".yellow());
            }
        }
    }
    Ok(())
}
