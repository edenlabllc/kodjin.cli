use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: CliCommand,
    /// Select which FHIR server to use.
    /// The default one will be used if not specified.
    #[clap(short, long)]
    pub server: Option<String>,
}

#[derive(Subcommand, Clone, Debug)]
pub enum CliCommand {
    /// Manage FHIR server URLs
    #[command(subcommand)]
    Server(ServerCommand),
    /// Show FHIR server metadata
    Metadata,
}

#[derive(Subcommand, Clone, Debug)]
pub enum ServerCommand {
    List,
    /// Add a new FHIR server with the provided URL
    Add {
        url: String,
        #[clap(short, long)]
        name: Option<String>,
    },
    /// Remove a FHIR server
    Remove {
        name: String,
    },
    /// Set a FHIR server as the default instance
    Default {
        name: String,
    },
}
