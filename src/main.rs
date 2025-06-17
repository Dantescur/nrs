mod commands;
mod config;
mod error;
mod utils;

use clap::{
    CommandFactory, FromArgMatches, Parser,
    builder::styling::{AnsiColor, Effects, Styles},
};
use clap_autocomplete::{add_subcommand, test_subcommand};
use commands::{
    Commands, add, current, doctor, edit, list, prune, remove, reset, show, test, use_cmd,
};
use config::Config;
use error::NrsError;

#[derive(Parser)]
#[command(name = "nrs", about = "Node.js registry switcher")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> Result<(), NrsError> {
    let styles = Styles::styled()
        .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .usage(AnsiColor::Green.on_default() | Effects::BOLD)
        .literal(AnsiColor::Cyan.on_default())
        .placeholder(AnsiColor::Magenta.on_default())
        .error(AnsiColor::Red.on_default() | Effects::BOLD)
        .valid(AnsiColor::Green.on_default() | Effects::BOLD)
        .invalid(AnsiColor::Red.on_default() | Effects::BOLD | Effects::UNDERLINE);

    let mut command = Cli::command().styles(styles);
    let command_clone = command.clone();

    command = add_subcommand(command);

    let matches = command.get_matches();

    if let Some(result) = test_subcommand(&matches, command_clone) {
        if let Err(err) = result {
            eprintln!("Error al instalar el autocompletado: {err}");
            std::process::exit(1);
        } else {
            std::process::exit(0);
        }
    }

    let cli = Cli::from_arg_matches(&matches)?;
    let config = Config::load()?;

    match cli.command {
        Commands::Ls { sort } => list(config, sort),
        Commands::Use {
            name,
            backup,
            local,
        } => use_cmd(config, &name, backup, local),
        Commands::Add { name, url } => add(config, &name, &url),
        Commands::Remove { name } => remove(config, &name),
        Commands::Reset { yes, all } => reset(config, yes, all),
        Commands::Prune { local, dry_run } => prune(config, local, dry_run),
        Commands::Doctor => doctor(config),
        Commands::Edit { name, new_url } => edit(config, &name, &new_url),
        Commands::Show { local } => show(local),
        Commands::Current { local } => current(config, local),
        Commands::Test { name, local } => test(config, &name, local),
    }
}
