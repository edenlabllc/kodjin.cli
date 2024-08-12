mod downloader;
mod package;
mod processor;

use crate::{client::FhirClient, registry::RegistryClient};
use anyhow::Context;
use console::style;
use deno_semver::package::PackageReq;
use futures::future::{try_join_all, BoxFuture};
use indexmap::IndexMap;
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use package::{FhirPackage, PackageIndex, PackageManifest};
use serde_json::Value;
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::Instant,
};
use tokio::sync::Semaphore;

#[derive(Clone, Copy)]
pub struct InstallContext<'a> {
    pub fhir_client: &'a FhirClient,
    pub action: Action,
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

pub fn install_package<'a>(
    ctx: InstallContext<'a>,
    registry_client: &'a RegistryClient,
    package: String,
    progress: &'a MultiProgress,
    current_packages: &'a Mutex<HashSet<String>>,
    semaphore: &'a Semaphore,
) -> BoxFuture<'a, anyhow::Result<()>> {
    // Avoid processing the same package twice
    if !current_packages.lock().unwrap().insert(package.clone()) {
        return Box::pin(async { Ok(()) });
    }

    let _permit = semaphore.acquire();

    Box::pin(async move {
        let package_req = PackageReq::from_str(&package).context("Invalid package request")?;

        let bar_style = ProgressStyle::with_template(&format!(
            "{{spinner}} {}: {{msg}}",
            style(&package_req).bold()
        ))
        .unwrap();
        let bar = ProgressBar::new_spinner()
            .with_message("Fetching package info")
            .with_style(bar_style.clone());
        let bar = progress.add(bar);

        let package_info = registry_client.package_info(&package_req.name).await?;
        let mut versions = package_info.versions.keys().collect::<Vec<_>>();
        versions.sort_unstable();

        let matching_version = versions
            .into_iter()
            .rev()
            .find(|version| package_req.version_req.matches(version))
            .with_context(|| "Could not find matching package \"{package}\"")?;
        let version_info = package_info.versions.get(matching_version).unwrap();

        let package = downloader::download_package(
            registry_client,
            package_req.name.clone(),
            version_info.clone(),
            bar.clone(),
        )
        .await?;

        bar.set_style(bar_style);
        bar.set_message("Waiting for dependencies");

        let manifest = package.read_manifest()?;

        let dependency_tasks = manifest.dependencies.iter().map(|(name, version)| {
            let package = format!("{name}@{version}");
            install_package(
                ctx,
                registry_client,
                package,
                progress,
                current_packages,
                semaphore,
            )
        });

        try_join_all(dependency_tasks).await?;

        let index = package.read_index()?;
        process_files_from_index(ctx, &package, manifest, index, &bar).await?;

        Ok(())
    })
}

async fn process_files_from_index(
    ctx: InstallContext<'_>,
    package: &FhirPackage,
    manifest: PackageManifest,
    index: PackageIndex,
    bar: &ProgressBar,
    // progress: &MultiProgress,
) -> anyhow::Result<()> {
    for file in index.files {
        bar.set_message(format!(
            "{} {} {}",
            ctx.action.bar_prefix(),
            file.resource_type,
            file.id
        ));

        let file_path = file.get_path();

        let full_file_path = package.dir.join(&file_path);
        let data = fs::read(full_file_path).with_context(|| {
            format!(
                "Could not read file {} from package {}",
                file_path.display(),
                manifest.name
            )
        })?;
        let data = serde_json::from_slice(&data).with_context(|| {
            format!(
                "Could not parse file {} from package {}",
                file_path.display(),
                manifest.name
            )
        })?;

        let resource = Resource {
            data,
            id: file.id,
            source_path: file_path.clone(),
        };

        match processor::process_resource(&file.resource_type, resource, ctx.fhir_client, bar).await
        {
            Ok(()) => (),
            Err(err) => {
                let msg = format!(
                    "{} could not process file \"{}\" in package {}: {err:#}",
                    style("Warning:").yellow(),
                    file_path.display(),
                    manifest.name
                );
                bar.suspend(|| {
                    println!("{msg}");
                })
            }
        }
    }

    Ok(())
}

pub async fn process_directory(ctx: InstallContext<'_>, root_path: &Path) -> anyhow::Result<()> {
    let started_at = Instant::now();

    let bar = ProgressBar::new_spinner().with_message("Loading data");

    // Grouped by resource type
    let mut resources: IndexMap<String, Vec<Resource>> = IndexMap::new();

    let paths = load_file_list(root_path)?;
    for file_path in paths {
        let relative_path = file_path.strip_prefix(root_path)?;
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

    let processed_count = processor::process_resources(ctx, resources).await;

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
            let subfiles = load_file_list(&entry.path())?;
            files.extend(subfiles);
        }
    }

    Ok(files)
}
