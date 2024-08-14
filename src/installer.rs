mod downloader;
mod package;
mod processor;
mod report;
mod resource;

const BASE_PACKAGE: &str = "hl7.fhir.r4.core";

use crate::{client::FhirClient, registry::RegistryClient};
use anyhow::Context;
use console::style;
use deno_npm::registry::{NpmPackageInfo, NpmPackageVersionInfo};
use deno_semver::package::PackageReq;
use futures::future::{try_join_all, BoxFuture};
use indexmap::IndexMap;
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use package::{FhirPackage, PackageIndexFile, PackageManifest};
use processor::PackageInstallStatus;
use report::InstallReport;
use resource::{Resource, ResourceInfo};
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
    // TODO: this should wait via a channel
    if !current_packages.lock().unwrap().insert(package.clone()) {
        return Box::pin(async { Ok(()) });
    }

    let _permit = semaphore.acquire();

    Box::pin(async move {
        let package_req = PackageReq::from_str(&package).context("Invalid package request")?;

        // Assume the base package is always installed
        if package_req.name == BASE_PACKAGE {
            return Ok(());
        }

        let bar_style = ProgressStyle::with_template(&format!(
            "{{spinner}} {}: {{msg}}",
            style(&package_req).bold()
        ))
        .unwrap();
        let bar = ProgressBar::new_spinner()
            .with_message("Fetching package info")
            .with_style(bar_style.clone());
        let bar = progress.add(bar);

        let (_package_info, version_info) =
            resolve_version_info(&package_req, registry_client).await?;

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

        bar.set_style(
            ProgressStyle::with_template(&format!(
                "{{spinner}} {}: {{wide_msg}} [{{pos}}/{{len}}]",
                style(&package_req).bold()
            ))
            .unwrap(),
        );

        let mut report = InstallReport::default();

        let install_status =
            processor::check_package_installed(&package, ctx.fhir_client, &bar).await?;

        if let PackageInstallStatus::NotInstalled(missing_files) = install_status {
            let index = package.read_index()?;

            bar.set_length(index.files.len() as u64);
            report.already_existed = index.files.len() - missing_files.len();

            process_files(ctx, &package, manifest, missing_files, &bar, &mut report).await?;

            bar.suspend(|| {
                println!(
                    "Installed package {} ({} resources created, {} errors, and {} already existed)", 
                    style(&package_req).bold(), 
                    style(report.created).bold(), 
                    style(report.errors).bold(), 
                    style(report.already_existed).bold()
                );
            })
        } else {
            bar.suspend(|| {
                println!(
                    "Package {} is already installed",
                    style(&package_req).bold()
                );
            })
        }

        Ok(())
    })
}

async fn resolve_version_info(
    package_req: &PackageReq,
    registry_client: &RegistryClient,
) -> anyhow::Result<(NpmPackageInfo, NpmPackageVersionInfo)> {
    let package_info = registry_client.package_info(&package_req.name).await?;
    let mut versions = package_info.versions.keys().collect::<Vec<_>>();
    versions.sort_unstable();

    let matching_version = versions
        .into_iter()
        .rev()
        .find(|version| package_req.version_req.matches(version))
        .with_context(|| "Could not find matching package \"{package}\"")?;
    let version_info = package_info.versions.get(matching_version).unwrap().clone();
    Ok((package_info, version_info))
}

async fn process_files(
    ctx: InstallContext<'_>,
    package: &FhirPackage,
    manifest: PackageManifest,
    files: Vec<PackageIndexFile>,
    bar: &ProgressBar,
    report: &mut InstallReport,
    // progress: &MultiProgress,
) -> anyhow::Result<()> {
    for file in files {
        bar.set_message(format!(
            "{} {} {}",
            ctx.action.bar_prefix(),
            file.resource_info.resource_type,
            file.resource_info.id
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
            info: file.resource_info.clone(),
            source_path: file_path.clone(),
        };

        match processor::process_resource(
            &file.resource_info.resource_type,
            resource,
            ctx.fhir_client,
            bar,
        )
        .await
        {
            Ok(()) => {
                report.created += 1;
            }
            Err(err) => {
                let path: PathBuf = file_path.components().skip(1).collect();

                let msg = format!(
                    "{} could not process file {} in package {}: {err:#}",
                    style("Warning:").yellow(),
                    style(path.display()).bold(),
                    style(&manifest.name).bold(),
                );
                bar.suspend(|| {
                    println!("{msg}");
                });
                report.errors += 1;
            }
        }
        bar.inc(1);
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
    let info: ResourceInfo = serde_json::from_str(&contents)?;
    let data: Value = serde_json::from_str(&contents)?;

    let resource_type = info.resource_type.clone();
    let resource = Resource {
        data,
        info,
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

pub async fn check_package_installed(
    ctx: InstallContext<'_>,
    registry_client: &RegistryClient,
    package: &str,
) -> anyhow::Result<()> {
    let package_req = PackageReq::from_str(package)?;

    let bar_style = ProgressStyle::with_template(&format!(
        "{{spinner}} {}: {{msg}}",
        style(&package_req).bold()
    ))
    .unwrap();
    let bar = ProgressBar::new_spinner()
        .with_message("Fetching package info")
        .with_style(bar_style.clone());

    let (_package_info, version_info) = resolve_version_info(&package_req, registry_client).await?;

    let fhir_package = downloader::download_package(
        registry_client,
        package_req.name.clone(),
        version_info,
        bar.clone(),
    )
    .await?;

    let progress = MultiProgress::new();
    let bar = progress.add(bar);
    bar.set_style(
        ProgressStyle::with_template("{spinner} [{pos}/{len}] {msg} [{wide_bar}]")
            .unwrap()
            .progress_chars("#>-"),
    );

    let status = processor::check_package_installed(&fhir_package, ctx.fhir_client, &bar).await?;
    progress.clear()?;

    match status {
        PackageInstallStatus::Installed => {
            println!(
                "Package {} is {}",
                style(&package_req).bold(),
                style("already installed").green()
            );
        }
        PackageInstallStatus::NotInstalled(missing) => {
            let index = fhir_package.read_index()?;

            let installed_text = if missing.len() == index.files.len() {
                style("not installed").red()
            } else {
                style("partially installed").yellow()
            };

            println!(
                "Package {} is {installed_text} ({}/{} resources missing)",
                style(&package_req).bold(),
                missing.len(),
                index.files.len(),
            );
            println!("The following files are missing:");

            // Group by resource type for better output first
            let mut resource_types: IndexMap<&str, Vec<&PackageIndexFile>> = IndexMap::new();
            for file in &missing {
                resource_types
                    .entry(&file.resource_info.resource_type)
                    .or_default()
                    .push(file);
            }

            for (resource_type, files) in resource_types {
                println!("{resource_type}:");

                for file in files {
                    if let Some(canonical_url) = file.resource_info.canonical_url() {
                        println!("- {} ({})", file.filename, style(canonical_url).bold());
                    } else {
                        println!("- {}", file.filename,);
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn print_tree<'a>( registry_client: &'a RegistryClient, package: &'a str, recursion_level: usize) -> BoxFuture<'a, anyhow::Result<()>> {
    Box::pin(async move {
        let package_req = PackageReq::from_str(package)?;

        let bar_style = ProgressStyle::with_template(&format!(
            "{{spinner}} {}: {{msg}}",
            style(&package_req).bold()
        ))
        .unwrap();
        let bar = ProgressBar::new_spinner()
            .with_message("Fetching package info")
            .with_style(bar_style.clone());

        let (_package_info, version_info) = resolve_version_info(&package_req, registry_client).await?;

        let fhir_package = downloader::download_package(
            registry_client,
            package_req.name.clone(),
            version_info.clone(),
            bar.clone(),
        )
        .await?;
        
        bar.finish_and_clear();
        
        let index = fhir_package.read_index()?;
        println!("{} - {} ({} resources)", " ".repeat(recursion_level * 2), style(format!("{}@{}", package_req.name, version_info.version)).bold(), index.files.len());
        
        let manifest = fhir_package.read_manifest()?;
        for (name, version) in manifest.dependencies {
            
            let package = format!("{name}@{version}");
            print_tree(registry_client, &package, recursion_level + 1).await?;
        }

        Ok(())
    })
}