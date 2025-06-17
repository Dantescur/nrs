use clap::{Subcommand, ValueHint};
use list::SortOrder;

mod add;
mod current;
mod doctor;
mod edit;
mod list;
mod prune;
mod remove;
mod reset;
mod show;
mod test;
mod r#use;

#[derive(Subcommand)]
pub enum Commands {
    /// List all registries
    Ls {
        #[arg(long, default_value = "name")]
        sort: SortOrder,
    },
    /// Show current registry
    Current {
        #[arg(long)]
        local: bool,
    },
    /// Use a specific registry
    Use {
        #[arg(value_hint = ValueHint::Other)]
        name: String,
        #[arg(long)]
        backup: bool,
        #[arg(long)]
        local: bool,
    },
    /// Add a new registry
    Add {
        #[arg(value_hint = ValueHint::Other)]
        name: String,
        #[arg(value_hint = ValueHint::Url)]
        url: String,
    },
    /// Remove a registry
    Remove {
        #[arg(value_hint = ValueHint::Other)]
        name: String,
    },
    /// Restore to defaults
    Reset {
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        all: bool,
    },
    /// Remove registries that are not reachable
    Prune {
        #[arg(long)]
        local: bool,
        #[arg(long)]
        dry_run: bool,
    },
    /// Check the environment and config for problems
    Doctor,
    /// Edit a registry
    Edit {
        #[arg(value_hint = ValueHint::Other)]
        name: String,
        #[arg(value_hint = ValueHint::Url)]
        new_url: String,
    },
    /// Show current npmrc
    Show {
        #[arg(long)]
        local: bool,
    },
    /// Test registry availability
    Test {
        #[arg(default_value = "", value_hint = ValueHint::Other)]
        name: String,
        #[arg(long)]
        local: bool,
    },
}

pub use add::execute as add;
pub use current::execute as current;
pub use doctor::execute as doctor;
pub use edit::execute as edit;
pub use list::execute as list;
pub use prune::execute as prune;
pub use remove::execute as remove;
pub use reset::execute as reset;
pub use show::execute as show;
pub use test::execute as test;
pub use r#use::execute as use_cmd;
