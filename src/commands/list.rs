use clap::ValueEnum;
use colored::*;

use crate::{config::Config, error::NrsError};

#[derive(Clone, Copy, ValueEnum)]
pub enum SortOrder {
    Name,
    Url,
    Default,
}

pub fn execute(config: Config, sort: SortOrder) -> Result<(), NrsError> {
    let mut registries: Vec<(String, String)> = config
        .registries
        .iter()
        .chain(config.custom_registries.iter())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    match sort {
        SortOrder::Name => registries.sort_by(|a, b| a.0.cmp(&b.0)),
        SortOrder::Url => registries.sort_by(|a, b| a.1.cmp(&b.1)),
        SortOrder::Default => {
            registries.sort_by(|a, b| {
                let a_idx = config
                    .registry_order
                    .iter()
                    .position(|n| n == &a.0)
                    .unwrap_or(usize::MAX);
                let b_idx = config
                    .registry_order
                    .iter()
                    .position(|n| n == &b.0)
                    .unwrap_or(usize::MAX);
                a_idx.cmp(&b_idx)
            });
        }
    }
    for (name, url) in registries {
        let marker = if config.current.as_deref() == Some(&name) {
            format!("{}", "🟢".green())
        } else {
            "".to_string()
        };
        let name_str = if config.current.as_deref() == Some(&name) {
            format!("{}", name.bold().green())
        } else {
            name.normal().white().to_string()
        };
        println!("{:2} {:15} {}", marker, name_str, url)
    }
    Ok(())
}
