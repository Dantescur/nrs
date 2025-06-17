use crate::{config::Config, error::NrsError};

pub fn execute(mut config: Config, name: &str, url: &str) -> Result<(), NrsError> {
    config.add_registry(name, url)?;
    println!("Added registry: {} ({})", name, url);
    Ok(())
}
