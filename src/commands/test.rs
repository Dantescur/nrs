use std::time::Instant;

use crate::{config::Config, error::NrsError, utils::get_local_registry_url};
use colored::Colorize;

pub fn execute(config: Config, name: &str, local: bool) -> Result<(), NrsError> {
    if local {
        let Some(url) = get_local_registry_url()? else {
            println!("{}", "No local .npmrc or registry found".yellow());
            return Ok(());
        };
        let client = reqwest::blocking::Client::new();
        let start = Instant::now();
        let response = client.head(&url).send();
        let duration = start.elapsed().as_millis();
        match response {
            Ok(resp) if resp.status().is_success() => {
                println!(
                    "{} {} ({}ms)",
                    "✅ Local registry is reachable:".green(),
                    url,
                    duration
                );
            }
            _ => {
                println!(
                    "{} {} ({}ms)",
                    "🔴 Local registry is NOT reachable:".red(),
                    url,
                    duration
                );
            }
        }
        Ok(())
    } else if name.is_empty() {
        let mut registries: Vec<_> = config
            .registries
            .keys()
            .chain(config.custom_registries.keys())
            .cloned()
            .collect();
        registries.sort();
        for name in registries {
            let (ok, duration) = config.test_registry(&name)?;
            let status = if ok {
                format!("{}", "✅ OK".green())
            } else {
                format!("{}", "🔴 Failed".red())
            };
            let time = duration.map(|d| format!(" ({}ms)", d)).unwrap_or_default();
            let current = if config.current.as_deref() == Some(&name) {
                format!("{}", "🟢".green()).to_string()
            } else {
                "".to_string()
            };
            let url = config
                .registries
                .get(&name)
                .or_else(|| config.custom_registries.get(&name))
                .unwrap();
            println!("{:2} {:15} {} - {}{}", current, name, status, url, time);
        }
        Ok(())
    } else {
        let (ok, duration) = config.test_registry(name)?;
        let status = if ok {
            format!("{}", "✅ OK".green())
        } else {
            format!("{}", "🔴 Failed".red())
        };
        let time = duration.map(|d| format!(" ({}ms)", d)).unwrap_or_default();
        let url = config
            .registries
            .get(name)
            .or_else(|| config.custom_registries.get(name))
            .ok_or_else(|| NrsError::RegistryNotFound(name.to_string()))?;
        println!(
            "{:2} {:15} {} - {}{}",
            if config.current.as_deref() == Some(name) {
                format!("{}", "🟢".green()).to_string()
            } else {
                "".to_string()
            },
            name,
            status,
            url,
            time
        );
        Ok(())
    }
}
