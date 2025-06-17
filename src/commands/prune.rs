use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    time::Instant,
};

use colored::Colorize;

use crate::{
    config::Config,
    error::NrsError,
    utils::{get_local_registry_url, get_npmrc_path, get_npmrc_path_local},
};

pub fn execute(mut config: Config, local: bool, dry_run: bool) -> Result<(), NrsError> {
    if local {
        let npmrc_path = get_npmrc_path_local(true)?;
        let local_url = get_local_registry_url()?;
        if let Some(url) = local_url {
            let client = reqwest::blocking::Client::new();
            let start = Instant::now();
            let response = client.head(&url).send();
            let duration = start.elapsed().as_millis();
            let reachable = matches!(response, Ok(resp) if resp.status().is_success());

            if reachable {
                println!(
                    "{} {} ({}ms)",
                    "✅ Local registry is reachable:".green(),
                    url,
                    duration
                );
            } else {
                println!(
                    "{} {} ({}ms)",
                    "🔴 Local registry is NOT reachable:".red(),
                    url,
                    duration
                );
                if !dry_run {
                    if npmrc_path.exists() {
                        let mut lines = Vec::new();
                        let file = File::open(&npmrc_path)?;
                        let reader = BufReader::new(file);
                        for line in reader.lines() {
                            let line = line?;
                            if !line.trim().starts_with("registry=") {
                                lines.push(line);
                            }
                        }
                        let content = if lines.is_empty() {
                            "".to_string()
                        } else {
                            lines.join("\n") + "\n"
                        };
                        if content.is_empty() {
                            fs::remove_file(&npmrc_path)?;
                            println!("Removed local .npmrc (no non-registry lines)");
                        } else {
                            fs::write(&npmrc_path, content)?;
                            println!("Removed registry from local .npmrc");
                        }
                    }
                } else {
                    println!("Dry-run: Would remove registry from local .npmrc");
                }
            }
        } else {
            println!("{}", "No registry found in local .npmrc".yellow());
        }
    } else {
        let mut unreachable = Vec::new();
        let mut registries: Vec<_> = config.custom_registries.keys().cloned().collect();
        registries.sort();
        for name in registries {
            let (ok, duration) = config.test_registry(&name)?;
            let time = duration.map(|d| format!(" ({}ms)", d)).unwrap_or_default();
            if !ok {
                unreachable.push(name.clone());
                println!(
                    "{} {} - {}{}",
                    "🔴 Unreachable:".red(),
                    name,
                    config.custom_registries[&name],
                    time
                );
            } else {
                println!(
                    "{} {} - {}{}",
                    "✅ Reachable:".green(),
                    name,
                    config.custom_registries[&name],
                    time
                );
            }
        }
        if unreachable.is_empty() {
            println!("{}", "All custom registries are reachable".green());
        } else if dry_run {
            println!(
                "{} Would remove {} unreachable custom registries: {}",
                "Dry-run:".yellow(),
                unreachable.len(),
                unreachable.join(", ")
            );
        } else {
            for name in &unreachable {
                config.remove_registry(name)?;
            }
            println!(
                "Removed {} unreachable custom registries: {}",
                unreachable.len(),
                unreachable.join(", ")
            );
            if config.current.is_none() {
                let npmrc_path = get_npmrc_path()?;
                if npmrc_path.exists() {
                    let mut lines = Vec::new();
                    let file = File::open(&npmrc_path)?;
                    let reader = BufReader::new(file);
                    for line in reader.lines() {
                        let line = line?;
                        if !line.trim().starts_with("registry=") {
                            lines.push(line);
                        }
                    }
                    let content = if lines.is_empty() {
                        "".to_string()
                    } else {
                        lines.join("\n") + "\n"
                    };
                    if content.is_empty() {
                        fs::remove_file(&npmrc_path)?;
                    } else {
                        fs::write(&npmrc_path, content)?;
                    }
                }
            }
        }
    }
    Ok(())
}
