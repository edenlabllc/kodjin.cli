use crate::{
    args::ServerCommand,
    client::FhirClient,
    config::{Config, ServerConfig},
};
use anyhow::bail;
use indicatif::ProgressBar;
use std::{io::Write, time::Duration};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn server(cmd: ServerCommand, mut config: Config) -> anyhow::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    match cmd {
        ServerCommand::List => {
            stdout.set_color(ColorSpec::new().set_bold(true).set_underline(true))?;
            writeln!(stdout, "List of currently configured servers:")?;
            stdout.reset()?;

            for (server_name, server_config) in &config.servers {
                write!(stdout, "- ")?;
                if *server_name == server_config.url {
                    stdout.set_color(ColorSpec::new().set_bold(true))?;
                    write!(stdout, "{server_name}")?;
                    stdout.reset()?;
                } else {
                    stdout.set_color(ColorSpec::new().set_bold(true))?;
                    write!(stdout, "{server_name}")?;
                    stdout.reset()?;
                    write!(stdout, " ({})", server_config.url)?;
                }

                if config.current_server.as_ref() == Some(server_name) {
                    stdout.set_color(ColorSpec::new().set_bold(true))?;
                    write!(stdout, " (default)")?;
                    stdout.reset()?;
                }
                writeln!(stdout)?;
            }

            Ok(())
        }
        ServerCommand::Add { url, name } => {
            let name = name.unwrap_or_else(|| url.clone());

            if config.servers.contains_key(&name) {
                bail!("Server {name} already exists");
            }

            let bar = ProgressBar::new_spinner().with_message(format!("Checking server {url}"));
            bar.enable_steady_tick(Duration::from_millis(100));

            let client = FhirClient::new(url.clone());
            let metadata = client.get_metadata()?;

            config.servers.insert(name.clone(), ServerConfig { url });

            if config.servers.len() == 1 {
                config.current_server = Some(config.servers.keys().next().unwrap().clone());
            }

            config.save()?;

            bar.finish_and_clear();

            write!(stdout, "Added server")?;
            stdout.set_color(ColorSpec::new().set_bold(true))?;
            write!(stdout, " {name}")?;
            stdout.reset()?;

            if let Some(software) = &metadata.software {
                write!(stdout, " running {}", software.name)?;
                if let Some(version) = &software.version {
                    write!(stdout, " {version}")?;
                }
            }
            writeln!(stdout)?;

            Ok(())
        }
        ServerCommand::Remove { name } => {
            if config.servers.shift_remove(&name).is_some() {
                config.save()?;
                write!(stdout, "Removed server")?;
                stdout.set_color(ColorSpec::new().set_bold(true))?;
                writeln!(stdout, " {name}")?;
                stdout.reset()?;

                Ok(())
            } else {
                bail!("Server {name} not in configuration");
            }
        }
        ServerCommand::Default { name } => {
            if config.servers.contains_key(&name) {
                writeln!(stdout, "Setting {name} as the default server")?;

                config.current_server = Some(name);
                config.save()?;
                Ok(())
            } else {
                bail!("Server {name} not found");
            }
        }
    }
}

pub fn metadata(client: FhirClient) -> anyhow::Result<()> {
    let bar = ProgressBar::new_spinner().with_message("Fetching metadata");
    bar.enable_steady_tick(Duration::from_millis(100));

    let metadata = client.get_metadata()?;

    bar.finish_and_clear();

    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    let values = [
        ("Name", metadata.name),
        ("Publisher", metadata.publisher),
        ("URL", metadata.url),
        ("Date", Some(metadata.date)),
        (
            "Software",
            metadata
                .software
                .as_ref()
                .map(|software| software.name.clone()),
        ),
        (
            "Software Version",
            metadata.software.and_then(|software| software.version),
        ),
        ("FHIR Version", Some(metadata.fhir_version)),
    ];

    for (key, value) in values {
        if let Some(value) = value {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
            write!(stdout, "{key}: ")?;
            stdout.reset()?;
            writeln!(stdout, "{value}")?;
        }
    }

    Ok(())
}
