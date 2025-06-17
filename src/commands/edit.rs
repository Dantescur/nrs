use crate::{config::Config, error::NrsError};

pub fn execute(mut config: Config, name: &str, new_url: &str) -> Result<(), NrsError> {
    config.edit_registry(name, new_url)?;
    println!("Edited registry: {} ({})", name, new_url);
    Ok(())
}
