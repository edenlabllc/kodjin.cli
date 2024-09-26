use super::report::InstallReport;
use std::{fmt, path::PathBuf};
use tokio::sync::watch;

#[derive(Debug)]
pub struct InstallProgress {
    pub state: InstallState,
    pub report: InstallReport,
    pub errors: Vec<ResourceError>,
}

#[derive(Debug)]
pub enum InstallState {
    InProgress(watch::Receiver<()>),
    Completed,
    Skipped,
}

impl fmt::Display for InstallState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            InstallState::InProgress(_) => "Unfinished",
            InstallState::Completed => "Completed",
            InstallState::Skipped => "Skipped",
        };
        text.fmt(f)
    }
}

#[derive(Debug)]
pub struct ResourceError {
    pub path: PathBuf,
    // pub error: anyhow::Error,
}
