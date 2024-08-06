mod processor;

use crate::client::FhirClient;
use anyhow::Context;
use console::style;
use indexmap::IndexMap;
use indicatif::{HumanDuration, ProgressBar};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

pub struct InstallContext {
    pub client: FhirClient,
    pub action: Action,
    pub root_path: PathBuf,
}

struct Resource {
    data: Value,
    id: String,
    /// Relative path where the resource was loaded from
    source_path: PathBuf,
}

#[derive(Clone, Copy)]
pub enum Action {
    Install,
    Uninstall,
}

impl Action {
    fn bar_prefix(&self) -> &str {
        match self {
            Action::Install => "Uploading",
            Action::Uninstall => "Deleting",
        }
    }
}

pub async fn process_directory(ctx: InstallContext) -> anyhow::Result<()> {
    let started_at = Instant::now();

    let bar = ProgressBar::new_spinner().with_message("Loading data");

    // Grouped by resource type
    let mut resources: IndexMap<String, Vec<Resource>> = IndexMap::new();

    let paths = load_file_list(&ctx.root_path)?;
    for file_path in paths {
        let relative_path = file_path.strip_prefix(&ctx.root_path)?;
        bar.set_message(format!("Reading file {}", relative_path.to_string_lossy()));

        if let Err(err) = load_file(&mut resources, &file_path, relative_path) {
            bar.suspend(|| {
                let msg = format!("Warning: could not process file {relative_path:?}: {err:#}");
                println!("{}", style(msg).yellow())
            })
        }
    }

    bar.finish_and_clear();
    let count: usize = resources.values().map(|resources| resources.len()).sum();

    println!("{} resources loaded", style(count).bold());

    let processed_count = processor::process_resources(&ctx, resources).await;

    println!(
        "Successfully processed {} resources in {}",
        style(processed_count).bold(),
        style(HumanDuration(started_at.elapsed())).bold()
    );

    Ok(())
}

fn load_file(
    resources: &mut IndexMap<String, Vec<Resource>>,
    path: &Path,
    source_path: &Path,
) -> anyhow::Result<()> {
    let contents = fs::read_to_string(path)?;
    let data: Value = serde_json::from_str(&contents)?;

    let resource_type = data
        .get("resourceType")
        .context("Resource has no \"resourceType\" field")?
        .as_str()
        .context("\"resourceType\" is not a string")?
        .to_owned();

    let id = data
        .get("id")
        .context("Resource has no id")?
        .as_str()
        .context("Resource id is not a string")?
        .to_owned();

    let resource = Resource {
        data,
        id,
        source_path: source_path.to_owned(),
    };

    resources.entry(resource_type).or_default().push(resource);

    Ok(())
}

fn load_file_list(path: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for result in fs::read_dir(path)? {
        let entry = result?;
        let metadata = entry.metadata()?;

        if metadata.is_file() || metadata.is_symlink() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.ends_with(".json") {
                files.push(entry.path());
            }
        } else if metadata.is_dir() {
            let subfiles = load_file_list(path)?;
            files.extend(subfiles);
        }
    }

    Ok(files)
}
