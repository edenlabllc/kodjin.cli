mod args;
mod client;
mod config;
mod installer;
mod registry;
mod subcommands;

use anyhow::Context;
use args::{Args, CliCommand};
use clap::Parser;
use client::FhirClient;
use config::Config;
use console::style;
use std::process::ExitCode;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let args = Args::parse();

    match run(args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            print_error(err);
            ExitCode::FAILURE
        }
    }
}

async fn run(args: Args) -> anyhow::Result<()> {
    let config = Config::load_or_create()?;

    match args.command.clone() {
        CliCommand::Server(cmd) => subcommands::server(cmd, config).await,
        CliCommand::Metadata => {
            let client = get_client(&config, &args)?;
            subcommands::metadata(client).await
        }
        CliCommand::Install(cmd) => {
            let client = get_client(&config, &args)?;
            subcommands::install(cmd, client).await
        }
        CliCommand::Uninstall(cmd) => {
            let client = get_client(&config, &args)?;
            subcommands::uninstall(cmd, client).await
        }
    }
}

fn print_error(error: anyhow::Error) {
    eprintln!("{} {error:#}", style("Error:").red());
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
