use crate::{config::Config, error::NrsError, utils::get_local_registry_url};
use colored::Colorize;

pub fn execute(config: Config, local: bool) -> Result<(), NrsError> {
    if local {
        match get_local_registry_url()? {
            Some(url) => {
                let registry_name = config
                    .registries
                    .iter()
                    .chain(config.custom_registries.iter())
                    .find(|(_, registry_url)| *registry_url == &url)
                    .map(|(name, _)| name.as_str())
                    .unwrap_or("(Unknown registry)");

                println!(
                    "{} {} ({}) {}",
                    "Current local registry:".bold(),
                    registry_name.green().bold(),
                    url,
                    "(local .npmrc)".dimmed()
                )
            }
            None => {
                println!("{}", "No registry found in local .npmrc".yellow())
            }
        }
    } else if let Some(current) = &config.current {
        let url = config
            .registries
            .get(current)
            .or_else(|| config.custom_registries.get(current))
            .unwrap();
        println!(
            "{} {} ({}) {}",
            "Current registry:".bold(),
            current.green().bold(),
            url,
            "(global)".dimmed()
        );
    } else {
        println!("{}", "No registry selected".yellow());
    }
    Ok(())
}
