use crate::{config::Config, error::NrsError, utils::get_npmrc_path};
use colored::Colorize;

pub fn execute(config: Config) -> Result<(), NrsError> {
    let npmrc_path = get_npmrc_path()?;
    if !npmrc_path.exists() {
        println!("{} {}", "Missing .npmrc:".red(), npmrc_path.display());
    } else {
        println!("{} {}", ".npmrc found:".green(), npmrc_path.display());
    }

    let total_registries = config.registries.len() + config.custom_registries.len();
    if total_registries == 0 {
        println!("{}", "No registries configured".red());
    } else {
        println!(
            "{} {} ({} default, {} custom)",
            "Total registries:".green(),
            total_registries,
            config.registries.len(),
            config.custom_registries.len()
        );
    }

    if let Some(current) = &config.current {
        println!("{} {}", "Current registry:".green(), current);
    } else {
        println!("{}", "No current registry set.".yellow());
    }
    Ok(())
}
