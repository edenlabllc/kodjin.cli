use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::{convert::Infallible, fmt, path::PathBuf, str::FromStr};

use crate::storage::logs_dir;

/// Kodjin management CLI
///
/// Usage examples:
/// $ kodjin-cli server add https://demo.kodjin.com/fhir
/// $ kodjin-cli metadata
/// $ kodjin-cli info de.gematik.epa
/// $ kodjin-cli install hl7.fhir.us.core@4.0.0
/// $ kodjin-cli --errors-output=directory install hl7.fhir.us.core@4.0.0
/// $ kodjin-cli --server=kodjin-demo check hl7.fhir.us.core@4.0.0
///
/// For full information, see --help for each subcommand.
#[derive(Parser, Debug)]
#[command(version, about, verbatim_doc_comment)]
pub struct Args {
    #[command(subcommand)]
    pub command: CliCommand,
    /// Select which FHIR server to use.
    /// The default one will be used if not specified.
    #[clap(short, long)]
    pub server: Option<String>,
    /// Additional header to be sent to the FHIR server.
    #[clap(short = 'H', long)]
    pub header: Vec<String>,
    /// Skip TLS certificate validation
    #[clap(long, default_value_t = false)]
    pub insecure_certificates: bool,
    /// Timeout for requests (in seconds)
    #[clap(long, default_value_t = 30)]
    pub request_timeout: u64,
    /// Where errors should be written to.
    ///
    /// Can be either `stderr` (default), `directory` for the default logs directory
    /// or a custom path.
    #[clap(long, default_value_t)]
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
    Download {
        /// Perform resource preprocessing that is normally done before installation
        ///
        /// Currently this does the following:
        ///
        /// - Generates new resource ids for canonical resources (ones that have a url and version present)
        ///
        /// - Generates snapshots for StructureDefinition resources where they are missing
        ///
        /// - Makes references to other profiles within the current package in StructureDefinition resources version-specific
        #[clap(long, default_value_t = false)]
        preprocess: bool,
        #[clap(flatten)]
        package_args: PackageCommand,
    },
    /// Generate command autocompletions
    GenerateCompletions(GenerateCompletions),
}

#[derive(Subcommand, Clone, Debug)]
pub enum ServerCommand {
    /// List currently configured servers
    List,
    /// Add a new FHIR server with the provided URL
    Add {
        url: String,
        /// Override the url used for get and search operations.
        /// By default, the main url is used.
        #[clap(short, long)]
        search_url: Option<String>,
        #[clap(short, long)]
        name: Option<String>,
    },
    /// Remove a FHIR server
    Remove { name: String },
    /// Set a FHIR server as the default
    Default { name: String },
}

#[derive(Parser, Clone, Debug)]
pub struct GenerateCompletions {
    #[arg(value_enum)]
    pub shell: Shell,
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
    /// Do not automatically install package dependencies
    #[clap(long, default_value_t = false)]
    pub skip_dependencies: bool,
    /// How many search requests can be performed in parallel when checking package files
    #[clap(long, default_value_t = 10)]
    pub parallel_search_requests: usize,
    /// Skip resource preprocessing.
    /// This can be useful if you want to e.g. keep original resource IDs.
    ///
    /// Currently preprocessing does the following:
    ///
    /// - Generates new resource ids for canonical resources (ones that have a url and version present)
    ///
    /// - Generates snapshots for StructureDefinition resources where they are missing
    ///
    /// - Makes references to other profiles within the current package in StructureDefinition resources version-specific
    #[clap(long, default_value_t = false)]
    pub skip_preprocessing: bool,
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

#[derive(Clone, Debug, Default)]
pub enum LogsOutput {
    #[default]
    Stderr,
    Directory,
    Custom(String),
}

impl LogsOutput {
    pub fn get_dir(&self) -> Option<PathBuf> {
        match self {
            LogsOutput::Stderr => None,
            LogsOutput::Directory => Some(logs_dir().expect("Could not get logs dir")),
            LogsOutput::Custom(path) => Some(PathBuf::from(path)),
        }
    }
}

impl FromStr for LogsOutput {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let out = match s {
            "stderr" => Self::Stderr,
            "directory" => Self::Directory,
            _ => LogsOutput::Custom(s.to_owned()),
        };
        Ok(out)
    }
}

impl fmt::Display for LogsOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LogsOutput::Stderr => "stderr",
            LogsOutput::Directory => "directory",
            LogsOutput::Custom(value) => value,
        };
        s.fmt(f)
    }
}
