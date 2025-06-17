use crate::{config::Config, error::NrsError};

pub fn execute(mut config: Config, name: &str, backup: bool, local: bool) -> Result<(), NrsError> {
    config.set_current(name, backup, local)?;
    println!(
        "Switched to registry: {} ({})",
        name,
        if local {
            ".npmrc (local)"
        } else {
            ".npmrc (global)"
        }
    );
    Ok(())
}
