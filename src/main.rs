mod args;
mod client;
mod config;
mod subcommands;

use anyhow::Context;
use args::{Args, CliCommand};
use clap::Parser;
use client::FhirClient;
use config::Config;
use std::{io::Write, process::ExitCode};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

fn main() -> ExitCode {
    let args = Args::parse();

    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            print_error(err);
            ExitCode::FAILURE
        }
    }
}

fn run(args: Args) -> anyhow::Result<()> {
    let config = Config::load_or_create()?;

    match args.command {
        CliCommand::Server(cmd) => subcommands::server(cmd, config),
        CliCommand::Metadata => {
            let client = get_client(&config, &args)?;
            subcommands::metadata(client)
        }
    }
}

fn print_error(error: anyhow::Error) {
    let mut stderr = StandardStream::stderr(ColorChoice::Auto);
    stderr
        .set_color(ColorSpec::new().set_fg(Some(Color::Red)))
        .expect("Failed to set terminal color");
    writeln!(stderr, "Error: {error:#}").unwrap();
    stderr.reset().expect("Could not reset color");
}

fn get_client(config: &Config, args: &Args) -> anyhow::Result<FhirClient> {
    let server_config = match &args.server {
        Some(requested_server) => config
            .servers
            .get(requested_server)
            .with_context(|| format!("Server {requested_server} not found in configuration"))?,
        None => config.get_current_server()?,
    };

    Ok(FhirClient::new(server_config.url.clone()))
}
