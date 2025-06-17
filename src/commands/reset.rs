use crate::{config::Config, error::NrsError};
use colored::Colorize;

pub fn execute(config: Config, yes: bool, all: bool) -> Result<(), NrsError> {
    if yes {
        let mut new_config = Config::new();
        if !all {
            new_config.custom_registries = config.custom_registries.clone();
            new_config.registry_order = new_config
                .registries
                .keys()
                .chain(new_config.custom_registries.keys())
                .cloned()
                .collect();
            new_config.registry_order.sort();
            new_config.current = config.current.clone();
        }
        new_config.save()?;
        println!("{}", "Reset complete".yellow());
    } else {
        println!("{}", "Use --yes to confirm reset.".red());
    }
    Ok(())
}
