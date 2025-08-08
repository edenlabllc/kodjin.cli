mod downloader;
mod package;
mod processor;
mod progress;
mod report;
mod resource;

const BASE_PACKAGE: &str = "hl7.fhir.r4.core";

use crate::{
    args::{ExistingResourceBehaviour, LogsOutput},
    client::{operation_outcome::OperationOutcome, FhirClient, FhirError},
    installer::processor::find_installed_resource,
    print_values_table,
    registry::RegistryClient,
};
use anyhow::{bail, Context};
use console::{strip_ansi_codes, style};
use deno_npm::registry::{NpmPackageInfo, NpmPackageVersionInfo};
use deno_semver::package::PackageReq;
use futures::{
    future::{try_join_all, BoxFuture},
    stream, StreamExt, TryStreamExt,
};
use indexmap::IndexMap;
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use package::{FhirPackage, PackageIndex, PackageIndexFile, PackageManifest};
use processor::PackageInstallStatus;
use progress::{InstallProgress, InstallState, ResourceError};
use report::InstallReport;
use resource::{Resource, ResourceInfo};
use serde::Serialize;
use serde_json::Value;
use serde_with::skip_serializing_none;
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Instant,
};
use tokio::sync::{watch, Semaphore};

#[derive(Clone, Copy)]
pub struct InstallContext<'a> {
    pub fhir_client: &'a FhirClient,
    pub action: Action,
    pub progress: &'a MultiProgress,
    pub packages_progress: &'a Mutex<HashMap<String, Arc<Mutex<InstallProgress>>>>,
    pub semaphore: &'a Semaphore,
    pub registry_client: &'a RegistryClient,
    pub skip_preprocessing: bool,
    pub skip_strict_reference_versions: bool,
    pub skip_dependencies: bool,
    pub parallel_search_requests: usize,
    pub existing_resources_behaviour: ExistingResourceBehaviour,
    pub errors_output: &'a LogsOutput,
    pub start_time: chrono::DateTime<chrono::Local>,
}

#[derive(Clone, Copy, Debug)]
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

pub fn install_package_by_name(
    ctx: InstallContext<'_>,
    package: String,
) -> BoxFuture<'_, anyhow::Result<()>> {
    Box::pin(async move {
        let maybe_result_rx = ctx
            .packages_progress
            .lock()
            .unwrap()
            .get(&package)
            .and_then(|status| match &status.lock().unwrap().state {
                InstallState::InProgress(rx) => Some(rx.clone()),
                _ => None,
            });
        if let Some(mut rx) = maybe_result_rx {
            let _ = rx.changed().await;
            return Ok(());
        }

        let (_result_tx, result_rx) = watch::channel(());

        let install_progress = Arc::new(Mutex::new(InstallProgress {
            state: InstallState::InProgress(result_rx),
            report: InstallReport::default(),
            full_name: package.clone(), // Placeholder name until it gets replaced with something that's guarnateed to have version info
            errors: vec![],
        }));

        ctx.packages_progress
            .lock()
            .unwrap()
            .insert(package.clone(), install_progress.clone());

        let _permit = ctx.semaphore.acquire();

        let package_req = PackageReq::from_str(&package).context("Invalid package request")?;

        // Assume the base package is always installed
        if package_req.name == BASE_PACKAGE {
            install_progress.lock().unwrap().state = InstallState::Skipped;
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
        let bar = ctx.progress.add(bar);

        let (_package_info, version_info) =
            resolve_version_info(&package_req, ctx.registry_client).await?;

        install_progress.lock().unwrap().full_name =
            format!("{}@{}", package_req.name, version_info.version);

        let package = downloader::download_package(
            ctx.registry_client,
            package_req.name.clone(),
            version_info.clone(),
            bar.clone(),
        )
        .await?;

        match ctx.action {
            Action::Install => install_package(ctx, package, install_progress, &bar).await,
            Action::Uninstall => uninstall_package(ctx, package, install_progress, &bar).await,
        }
    })
}

fn install_package<'a>(
    ctx: InstallContext<'a>,
    package: FhirPackage,
    current_progress: Arc<Mutex<InstallProgress>>,
    bar: &'a ProgressBar,
) -> BoxFuture<'a, anyhow::Result<()>> {
    Box::pin(async move {
        let manifest = package.read_manifest()?;
        let package_req = format!("{}@{}", manifest.name, manifest.version);

        let bar_style = ProgressStyle::with_template(&format!(
            "{{spinner}} {}: {{msg}}",
            style(&package_req).bold(),
        ))
        .unwrap();
        bar.set_style(bar_style);
        bar.set_message("Waiting for dependencies");

        if !ctx.skip_dependencies {
            let dependency_tasks = manifest.dependencies.iter().map(|(name, version)| {
                let package = format!("{name}@{version}");
                let dependency_ctx = InstallContext {
                    existing_resources_behaviour: ExistingResourceBehaviour::Skip,
                    ..ctx
                };
                install_package_by_name(dependency_ctx, package)
            });

            try_join_all(dependency_tasks).await?;

            bar.set_style(
                ProgressStyle::with_template(&format!(
                    "{{spinner}} {}: {{wide_msg}} [{{pos}}/{{len}}]",
                    style(&package_req).bold()
                ))
                .unwrap(),
            );
        }

        let install_status = match ctx.existing_resources_behaviour {
            ExistingResourceBehaviour::Skip => {
                processor::check_package_installed(
                    &package,
                    ctx.fhir_client,
                    bar,
                    ctx.parallel_search_requests,
                )
                .await?
            }
            ExistingResourceBehaviour::Overwrite => {
                let index = package.read_index()?;
                PackageInstallStatus::NotInstalled(index.files)
            }
        };
        bar.reset();

        if let PackageInstallStatus::NotInstalled(missing_files) = install_status {
            bar.suspend(|| {
                println!(
                    "{}: installing {} resources",
                    style(&package_req).bold(),
                    missing_files.len()
                )
            });

            let index = package.read_index()?;

            bar.set_length(index.files.len() as u64);
            current_progress.lock().unwrap().report.already_existed =
                index.files.len() - missing_files.len();

            process_package_files(
                ctx,
                &package,
                manifest,
                missing_files,
                &index,
                bar,
                &current_progress,
            )
            .await?;
        }

        current_progress.lock().unwrap().state = InstallState::Completed;

        Ok(())
    })
}

async fn uninstall_package<'a>(
    ctx: InstallContext<'a>,
    package: FhirPackage,
    current_progress: Arc<Mutex<InstallProgress>>,
    bar: &'a ProgressBar,
) -> anyhow::Result<()> {
    let manifest = package.read_manifest()?;
    let package_req = format!("{}@{}", manifest.name, manifest.version);

    bar.set_style(
        ProgressStyle::with_template(&format!(
            "{{spinner}} {}: {{wide_msg}} [{{pos}}/{{len}}]",
            style(&package_req).bold()
        ))
        .unwrap(),
    );

    let index = package.read_index()?;

    bar.reset();
    bar.set_length(index.files.len() as u64);
    bar.set_message("Checking resources");

    let requests = index.files.into_iter().map(|file| async move {
        let id = find_installed_resource(ctx.fhir_client, &file.resource_info).await?;

        bar.inc(1);

        anyhow::Ok((file, id))
    });

    let existing = stream::iter(requests)
        .buffer_unordered(ctx.parallel_search_requests)
        .filter_map(|result| async {
            match result {
                Ok((info, Some(id))) => Some(Ok((info, id))),
                Ok((_, None)) => None,
                Err(err) => Some(Err(err)),
            }
        })
        .try_collect::<Vec<_>>()
        .await?;

    bar.reset();

    println!("Found {} resources to remove", style(existing.len()).bold());

    for (file, id) in existing {
        bar.set_message(format!(
            "{} {} {id}",
            ctx.action.bar_prefix(),
            file.resource_info.resource_type,
        ));

        match ctx
            .fhir_client
            .delete(&file.resource_info.resource_type, &id)
            .await
        {
            Ok(()) => {
                current_progress.lock().unwrap().report.created += 1;
            }
            Err(error) => {
                let file_path = file.get_path();
                let full_name = format!("{}@{}", manifest.name, manifest.version);

                log_resource_error(
                    ctx,
                    error,
                    &file_path,
                    &full_name,
                    bar,
                    Some(&file.resource_info),
                );

                let path: PathBuf = file_path.components().skip(1).collect();

                current_progress
                    .lock()
                    .unwrap()
                    .errors
                    .push(ResourceError { path });
                current_progress.lock().unwrap().report.errors += 1;
            }
        }
        bar.inc(1);
    }
    current_progress.lock().unwrap().state = InstallState::Completed;

    Ok(())
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
        .with_context(|| format!("Could not find matching package \"{package_req}\""))?;
    let version_info = package_info.versions.get(matching_version).unwrap().clone();
    Ok((package_info, version_info))
}

async fn process_package_files(
    ctx: InstallContext<'_>,
    package: &FhirPackage,
    manifest: PackageManifest,
    files: Vec<PackageIndexFile>,
    current_index: &PackageIndex,
    bar: &ProgressBar,
    current_progress: &Mutex<InstallProgress>,
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

        let resource_info = resource.info.clone();

        let full_name = format!("{}@{}", manifest.name, manifest.version);
        match processor::process_resource(
            ctx,
            &file.resource_info.resource_type,
            resource,
            current_index,
            &full_name,
            bar,
            None,
        )
        .await
        {
            Ok(()) => {
                current_progress.lock().unwrap().report.created += 1;
            }
            Err(error) => {
                let path: PathBuf = file_path.components().skip(1).collect();

                log_resource_error(
                    ctx,
                    error,
                    &file_path,
                    &full_name,
                    bar,
                    Some(&resource_info),
                );

                current_progress
                    .lock()
                    .unwrap()
                    .errors
                    .push(ResourceError { path });
                current_progress.lock().unwrap().report.errors += 1;
            }
        }
        bar.inc(1);
    }

    Ok(())
}

pub async fn process_directory(ctx: InstallContext<'_>, root_path: &Path) -> anyhow::Result<()> {
    let mut root_path = root_path.to_owned();

    let pkg_name = fs::canonicalize(&root_path)
        .context("Invalid path provided")?
        .file_name()
        .context("Provided path has no name")?
        .to_string_lossy()
        .into_owned();

    let started_at = Instant::now();

    let (_tx, rx) = watch::channel(());
    let current_progress = Arc::new(Mutex::new(InstallProgress {
        state: InstallState::InProgress(rx),
        report: InstallReport::default(),
        errors: vec![],
        full_name: pkg_name.clone(),
    }));

    ctx.packages_progress
        .lock()
        .unwrap()
        .insert(pkg_name.clone(), current_progress.clone());

    let bar = ProgressBar::new_spinner().with_message("Loading data");

    if root_path.join("package.json").exists() {
        let full_path = fs::canonicalize(&root_path).context("Could not resolve directory")?;
        if full_path
            .file_name()
            .is_some_and(|name| name.to_str() == Some("package"))
        {
            root_path = full_path
                .parent()
                .context("Could not get directory parent")?
                .to_path_buf();
        } else {
            bar.suspend(|| {
                println!("package.json file exists, but current directory is not 'package' - renaming FHIR package directories is not allowed due to file paths being used inside of the package.");
            });
        }
    }

    if root_path.join("package").join("package.json").exists() {
        bar.suspend(|| {
            println!("Found package.json file, processing as FHIR package");
        });

        let package = FhirPackage::new(root_path.clone());

        install_package(ctx, package, current_progress, &bar).await
    } else {
        bar.suspend(|| {
            println!("No package.json file found, processing as a basic directory");
        });

        // Grouped by resource type
        let mut resources: IndexMap<String, Vec<Resource>> = IndexMap::new();

        let paths = load_file_list(&root_path)?;
        for file_path in paths {
            let relative_path = file_path.strip_prefix(&root_path)?;
            bar.set_message(format!("Reading file {}", relative_path.to_string_lossy()));

            if let Err(error) = load_file(&mut resources, &file_path, relative_path) {
                log_resource_error(
                    ctx,
                    FhirError::Other(error),
                    &file_path,
                    &pkg_name,
                    &bar,
                    None,
                );
            }
        }

        bar.finish_and_clear();
        let count: usize = resources.values().map(|resources| resources.len()).sum();

        println!("{} resources loaded", style(count).bold());

        let processed_count =
            processor::process_directory_resources(ctx, resources, &current_progress, &pkg_name)
                .await;

        println!(
            "Processed {} resources in {}",
            style(processed_count).bold(),
            style(HumanDuration(started_at.elapsed())).bold()
        );

        if let Some(dir) = ctx.errors_output.get_dir() {
            println!(
                "Check {} for full error info",
                package_log_file(&dir, &pkg_name, ctx.start_time).display(),
            );
        }

        Ok(())
    }
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
    fhir_client: &FhirClient,
    registry_client: &RegistryClient,
    package: &str,
    parallel_search_requests: usize,
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
        ProgressStyle::with_template("{spinner} {msg} [{wide_bar}] [{pos}/{len}]")
            .unwrap()
            .progress_chars("#>-"),
    );

    let status = processor::check_package_installed(
        &fhir_package,
        fhir_client,
        &bar,
        parallel_search_requests,
    )
    .await?;
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
                        println!("  - {} ({})", file.filename, style(canonical_url).bold());
                    } else {
                        println!("  - {}", file.filename,);
                    }
                }
            }

            println!(
                "Package {} is {installed_text} ({}/{} resources present)",
                style(&package_req).bold(),
                index.files.len() - missing.len(),
                index.files.len(),
            );
        }
    }

    Ok(())
}

pub fn print_tree<'a>(
    registry_client: &'a RegistryClient,
    package: &'a str,
    recursion_level: usize,
) -> BoxFuture<'a, anyhow::Result<()>> {
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

        let version_info = match resolve_version_info(&package_req, registry_client).await {
            Ok((_package_info, version_info)) => version_info,
            Err(err) => {
                println!(
                    "{} - {} ({})",
                    " ".repeat(recursion_level * 2),
                    style(package_req).bold(),
                    style(format!("Could not resolve: {err:#}")).red()
                );
                return Ok(());
            }
        };

        let fhir_package = downloader::download_package(
            registry_client,
            package_req.name.clone(),
            version_info.clone(),
            bar.clone(),
        )
        .await?;

        bar.finish_and_clear();

        let index = fhir_package.read_index()?;
        println!(
            "{} - {} ({} resources)",
            " ".repeat(recursion_level * 2),
            style(format!("{}@{}", package_req.name, version_info.version)).bold(),
            index.files.len()
        );

        let manifest = fhir_package.read_manifest()?;
        for (name, version) in manifest.dependencies {
            let package = format!("{name}@{version}");
            print_tree(registry_client, &package, recursion_level + 1)
                .await
                .with_context(|| format!("Could not process dependency {package}"))?;
        }

        Ok(())
    })
}

pub async fn info(registry_client: &RegistryClient, package: &str) -> anyhow::Result<()> {
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

    let manifest = fhir_package.read_manifest()?;
    let index = fhir_package.read_index()?;

    let mut dependencies = manifest
        .dependencies
        .iter()
        .map(|(key, value)| format!("{key}@{value}"))
        .collect::<Vec<_>>();
    dependencies.sort();

    let mut resources_count: IndexMap<String, usize> = IndexMap::new();
    for file in index.files {
        *resources_count
            .entry(file.resource_info.resource_type)
            .or_default() += 1;
    }
    let mut count_text = resources_count
        .into_iter()
        .map(|(resource_type, count)| format!("{resource_type}: {count}"))
        .collect::<Vec<_>>();
    count_text.sort();

    let values = vec![
        ("Name", Some(manifest.name)),
        ("Version", Some(manifest.version)),
        ("Author", manifest.author),
        ("Description", manifest.description),
        (
            "FHIR Versions",
            if manifest.fhir_versions.is_empty() {
                None
            } else {
                Some(manifest.fhir_versions.join(", "))
            },
        ),
        ("Dependencies", Some(dependencies.join(", "))),
        ("Contents", Some(count_text.join(", "))),
    ];

    print_values_table(&values);

    Ok(())
}

pub async fn download(
    registry_client: &RegistryClient,
    package: &str,
    fhir_client: FhirClient,
    skip_strict_reference_versions: bool,
    preprocess: bool,
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

    let output_folder = format!("{}@{}", package_req.name, version_info.version);
    fs::create_dir_all(&output_folder)?;
    let full_final_path =
        fs::canonicalize(&output_folder).context("Could not get output directory")?;

    if fs::read_dir(&full_final_path)
        .is_ok_and(|mut dir| dir.next().is_some_and(|result| result.is_ok()))
    {
        bail!("Target directory already exists and is not empty");
    }

    let temp_output_path = full_final_path
        .parent()
        .context("Could not get parent directory")?
        .join(format!(".TEMP-{output_folder}"));

    let fhir_package = downloader::download_package_to(
        registry_client,
        package_req.name.clone(),
        version_info.clone(),
        bar.clone(),
        full_final_path,
        temp_output_path,
    )
    .await?;

    bar.set_message("Copying files");

    let index = fhir_package.read_index()?;

    for file in &index.files {
        let file_path = fhir_package.dir.join(file.get_path());
        let file_contents =
            fs::read_to_string(&file_path).context("Failed to read file in package")?;
        let resource_data: Value =
            serde_json::from_str(&file_contents).context("Failed to parse file")?;
        let mut resource = Resource {
            data: resource_data,
            info: file.resource_info.clone(),
            source_path: file.get_path(),
        };

        if preprocess {
            let changed = processor::preprocess_resource(
                &mut resource,
                &fhir_client,
                skip_strict_reference_versions,
                &file.resource_info.resource_type,
                &index,
                &bar,
            )
            .await
            .with_context(|| {
                format!(
                    "Could not preprocess resource {}",
                    file.get_path().display()
                )
            })?;

            if changed {
                let new_file_contents = serde_json::to_string_pretty(&resource.data)?;
                fs::write(&file_path, new_file_contents)
                    .with_context(|| format!("Could not write to {}", file_path.display()))?;

                bar.suspend(|| {
                    println!(
                        "Preprocessed file {}",
                        style(file.get_path().display()).bold()
                    );
                })
            }
        }
    }

    bar.finish_and_clear();
    println!(
        "Package downloaded to {}",
        style(fhir_package.dir.display()).bold()
    );

    Ok(())
}

pub fn print_report(ctx: InstallContext<'_>, primary_package: &str) {
    let mut total_errors = 0;

    let mut progress = ctx.packages_progress.lock().unwrap();
    println!("{:?} report:", ctx.action);
    if let Some(status) = progress.remove(primary_package) {
        let status = status.lock().unwrap();
        println!(
            "{}: {} ({})",
            style(primary_package).bold(),
            status.state,
            status.report.to_string(ctx.action)
        );

        if !status.errors.is_empty() {
            total_errors += status.errors.len();
            println!("{}", style("Failed resources:").red());

            for error in &status.errors {
                println!("- {}", style(error.path.display()).bold());
            }

            if let Some(dir) = ctx.errors_output.get_dir() {
                println!(
                    "Check {} for full error info",
                    package_log_file(&dir, &status.full_name, ctx.start_time).display(),
                );
            }
        }
    }

    if !progress.is_empty() {
        println!("Dependencies:");

        for (name, status) in progress.iter() {
            let status = status.lock().unwrap();
            println!(
                "- {}: {} ({})",
                style(name).bold(),
                status.state,
                status.report.to_string(ctx.action)
            );

            total_errors += status.errors.len();
            if !status.errors.is_empty() {
                println!("  {}", style("Failed resources:").red());

                for error in &status.errors {
                    println!("  - {}", style(error.path.display()).bold());
                }

                if let Some(dir) = ctx.errors_output.get_dir() {
                    println!(
                        "  Check {} for full error info",
                        package_log_file(&dir, &status.full_name, ctx.start_time).display(),
                    );
                }
            }
        }
    }

    if total_errors > 0 {
        if let LogsOutput::Stderr = ctx.errors_output {
            println!("Check earlier logs for error info");
        }
    }
}

fn log_resource_error(
    ctx: InstallContext<'_>,
    error: FhirError,
    file_path: &Path,
    pkg_name: &str,
    bar: &ProgressBar,
    resource_info: Option<&ResourceInfo>,
) {
    match ctx.errors_output {
        LogsOutput::Stderr => bar.suspend(|| {
            eprintln!(
                "{}: could not process file {} in package {}: {error}",
                style("Warning").yellow(),
                style(file_path.display()).bold(),
                style(&pkg_name).bold(),
            )
        }),
        LogsOutput::Directory | LogsOutput::Custom(_) => {
            let logs_dir = ctx.errors_output.get_dir().unwrap();
            // TODO: keep the file open with buffered writes
            let path = package_log_file(&logs_dir, pkg_name, ctx.start_time);
            match std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(path)
            {
                Ok(mut file) => {
                    let mut msg = JsonLogMessage {
                        package: pkg_name,
                        file: file_path,
                        error: None,
                        status_code: None,
                        outcome: None,
                        url: None,
                        version: None,
                        id: None,
                    };

                    match &error {
                        FhirError::Outcome {
                            status,
                            outcome,
                            url,
                        } => {
                            msg.status_code = Some(status.as_u16());
                            msg.outcome = Some(outcome);
                            msg.url = Some(url.as_str());
                        }
                        FhirError::Other(_) => {
                            msg.error = Some(strip_ansi_codes(&error.to_string()).into_owned());
                        }
                    }

                    if let Some(info) = resource_info {
                        msg.id = Some(&info.id);
                        msg.url = info.url.as_deref().or(msg.url);
                        msg.version = info.version.as_deref();
                    }

                    let log_contents = serde_json::to_string(&msg).unwrap();

                    if let Err(err) = writeln!(file, "{}", log_contents) {
                        eprintln!("Could not write log to file: {err}");
                    }
                }
                Err(err) => {
                    eprintln!(
                        "{} could not open logs file for writing: {err}",
                        style("Error:").red()
                    );
                }
            }
        }
    }
}

fn package_log_file(
    logs_dir: &Path,
    pkg_name: &str,
    start_time: chrono::DateTime<chrono::Local>,
) -> PathBuf {
    let file_name = format!(
        "{}-{}.ndjson",
        pkg_name.replace('@', "-"),
        start_time.naive_local().format("%F-%H-%M-%S")
    );
    logs_dir.join(file_name)
}

#[skip_serializing_none]
#[derive(Serialize)]
struct JsonLogMessage<'a> {
    pub package: &'a str,
    pub file: &'a Path,
    pub error: Option<String>,
    pub status_code: Option<u16>,
    pub outcome: Option<&'a OperationOutcome>,
    pub url: Option<&'a str>,
    pub version: Option<&'a str>,
    pub id: Option<&'a str>,
}
