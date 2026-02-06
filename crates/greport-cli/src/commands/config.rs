//! Configuration command handlers

use crate::args::ConfigCommands;
use crate::config::{create_default_config, default_config_path, load_config};

pub fn handle_config(command: &ConfigCommands) -> anyhow::Result<()> {
    match command {
        ConfigCommands::Show => {
            let config = load_config(None)?;
            println!("{}", toml::to_string_pretty(&config)?);
        }

        ConfigCommands::Init { force } => {
            let path = create_default_config(*force)?;
            println!("Configuration file created at: {}", path.display());
        }

        ConfigCommands::Set { key, value } => {
            println!("Setting {} = {}", key, value);
            // TODO: Implement config modification
            anyhow::bail!(
                "Config modification not yet implemented. Edit the config file directly."
            );
        }

        ConfigCommands::Path => {
            let path = default_config_path()?;
            println!("{}", path.display());
        }
    }

    Ok(())
}
