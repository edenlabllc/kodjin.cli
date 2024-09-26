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
    /// Timeout for requests (in seconds)
    #[clap(long, default_value_t = 30)]
    pub request_timeout: u64,
    /// Where errors should be written to
    #[clap(long, value_enum, default_value_t)]
    pub errors_output: LogsOutput,
}

#[derive(Subcommand, Clone, Debug)]
pub enum CliCommand {
    /// Manage FHIR server URLs
    #[command(subcommand)]
    Server(ServerCommand),
    /// Show FHIR server metadata
    Metadata,
    /// Install a FHIR package
    Install(PackageCommand),
    /// Uninstall a FHIR package
    #[command(alias = "remove")]
    Uninstall(PackageCommand),
    /// Check if a FHIR package is installed
    Check(PackageCommand),
    /// Print dependency tree of a FHIR package
    Tree(PackageCommand),
    /// Show information about a FHIR package
    Info(PackageCommand),
    /// Download a package locally
    Download(PackageCommand),
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
pub struct PackageCommand {
    /// Item to process
    pub name: String,
    /// Type of the item
    #[clap(value_enum, short, long, default_value_t)]
    pub r#type: InstallType,
    /// Registry URL for FHIR packages
    #[clap(short, long, default_value = "https://packages.simplifier.net")]
    pub registry: String,
    /// What should be done with resources that already exist
    ///
    /// Note: this setting is not applied to dependencies in order to avoid accidentally overwriting resources.
    #[clap(value_enum, long, default_value_t)]
    pub existing_resources: ExistingResourceBehaviour,
    /// Do not change profile references to be version-specific,
    /// keep them as-is instead
    #[clap(long, default_value_t = false)]
    pub skip_strict_reference_versions: bool,
    /// Perform resource preprocessing that is normally done before installation
    ///
    /// Currently does the following:
    /// - Generates new resource ids for canonical resources (ones that have a url and version present)
    /// - Generates snapshots for StructureDefinition resources where they are missing
    /// - Makes references to other profiles within the current package in StructureDefinition resources version-specific
    #[clap(long, default_value_t = false)]
    pub preprocess: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug, Default)]
pub enum ExistingResourceBehaviour {
    /// Skip existing resources
    #[default]
    Skip,
    /// Overwrite existing resources
    Overwrite,
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

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum LogsOutput {
    #[default]
    Stderr,
    #[value(alias("dir"))]
    Directory,
}
