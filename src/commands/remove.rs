use crate::{config::Config, error::NrsError};

pub fn execute(mut config: Config, name: &str) -> Result<(), NrsError> {
    config.remove_registry(&name)?;
    println!("Removed registry: {}", name);
    Ok(())
}
