use std::fs;

use colored::Colorize;

use crate::{error::NrsError, utils::get_npmrc_path_local};

pub fn execute(local: bool) -> Result<(), NrsError> {
    let npmrc_path = get_npmrc_path_local(local)?;
    if npmrc_path.exists() {
        let content = fs::read_to_string(npmrc_path)?;
        println!("{}", content);
    } else {
        println!("{}", "No .npmrc file found".yellow());
    }
    Ok(())
}
