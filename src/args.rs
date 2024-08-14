use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: CliCommand,
    /// Select which FHIR server to use.
    /// The default one will be used if not specified.
    #[clap(short, long)]
    pub server: Option<String>,
    /// Skip TLS certificate validation
    #[clap(long, default_value_t = false)]
    pub insecure_certificates: bool,
}

#[derive(Subcommand, Clone, Debug)]
pub enum CliCommand {
    /// Manage FHIR server URLs
    #[command(subcommand)]
    Server(ServerCommand),
    /// Show FHIR server metadata
    Metadata,
    /// Install a FHIR package
    Install(InstallCommand),
    /// Uninstall a FHIR package
    #[command(alias = "remove")]
    Uninstall(InstallCommand),
    /// Check if a FHIR package is installed
    Check(InstallCommand),
    /// Print dependency tree of a FHIR package
    Tree(InstallCommand),
    /// Show information about a FHIR package
    Info(InstallCommand),
}

#[derive(Subcommand, Clone, Debug)]
pub enum ServerCommand {
    /// List currently configured servers
    List,
    /// Add a new FHIR server with the provided URL
    Add {
        url: String,
        #[clap(short, long)]
        name: Option<String>,
    },
    /// Remove a FHIR server
    Remove { name: String },
    /// Set a FHIR server as the default
    Default { name: String },
}

#[derive(Parser, Clone, Debug)]
pub struct InstallCommand {
    /// Item to install
    pub name: String,
    /// Type of the installation target
    #[clap(value_enum, short, long, default_value_t)]
    pub r#type: InstallType,
    /// Registry URL for FHIR packages
    #[clap(short, long, default_value = "https://packages.simplifier.net")]
    pub registry: String,
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum InstallType {
    /// FHIR Package from a registry
    #[default]
    Package,
    /// Local directory
    #[value(alias("local"), alias("dir"))]
    Directory,
}
