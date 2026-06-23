pub mod resolver;

use super::{
    package::{PackageIndex, PackageIndexFile},
    resource::{Resource, ResourceInfo},
    Action, PackageContext,
};
use crate::{
    args::{ExistingResourceBehaviour, InstallType},
    client::{FhirClient, FhirError},
    installer::{
        log_resource_error, print_check_status,
        processor::resolver::sort_resources_by_dependencies,
        progress::{InstallProgress, InstallState},
        resource::is_resource_changed,
        ErrorsWriter,
    },
};
use anyhow::{anyhow, Context};
use console::style;
use futures::{stream, StreamExt, TryStreamExt};
use indexmap::IndexMap;
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;
use std::{path::Path, sync::Mutex, time::Duration};
use uuid::Uuid;

const RESOURCE_TYPES_ORDER: &[&str] = &[
    "StructureDefinition",
    "SearchParameter",
    "CodeSystem",
    "ValueSet",
    "ConceptMap",
];

pub async fn process_directory_resources(
    ctx: &PackageContext<'_>,
    mut resources: IndexMap<String, Vec<Resource>>,
    current_progress: &Mutex<InstallProgress>,
    name: &str,
) -> anyhow::Result<usize> {
    let count: usize = resources.values().map(|resources| resources.len()).sum();

    let bar = ProgressBar::new(count as u64)
        .with_message(format!("{} resources", ctx.action.bar_prefix()));
    bar.enable_steady_tick(Duration::from_millis(100));
    bar.set_style(ProgressStyle::with_template("{spinner} [{pos}/{len}] {msg}").unwrap());

    let mut processed_count = 0;
    let mut missing_resources = Vec::new();

    // First we process resources in the defined order
    for resource_type in RESOURCE_TYPES_ORDER {
        if let Some(resources) = resources.shift_remove(*resource_type) {
            processed_count += process_resources_type(
                ctx,
                resource_type,
                resources,
                &bar,
                current_progress,
                name,
                &mut missing_resources,
            )
            .await?;
        }
    }

    // Process remaining resource types which were not in the list
    for (resource_type, resources) in resources.into_iter() {
        processed_count += process_resources_type(
            ctx,
            &resource_type,
            resources,
            &bar,
            current_progress,
            name,
            &mut missing_resources,
        )
        .await?;
    }

    current_progress.lock().unwrap().state = InstallState::Completed;
    bar.finish_and_clear();

    if ctx.action == Action::Check {
        let mut grouped_resources: IndexMap<&str, Vec<_>> = IndexMap::new();
        for resource in &missing_resources {
            grouped_resources
                .entry(&resource.info.resource_type)
                .or_default()
                .push((
                    &resource.info,
                    resource
                        .source_path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("<Invalid filename>"),
                ));
        }

        let existing = current_progress.lock().unwrap().report.already_existed;

        print_check_status(
            name,
            existing,
            processed_count,
            InstallType::Directory,
            &grouped_resources,
        );
    }

    Ok(processed_count)
}

/// Returns the number of resources which were successfully uploaded
async fn process_resources_type(
    ctx: &PackageContext<'_>,
    resource_type: &str,
    resources: Vec<Resource>,
    bar: &ProgressBar,
    current_progress: &Mutex<InstallProgress>,
    pkg_name: &str,
    missing_resources: &mut Vec<Resource>,
) -> anyhow::Result<usize> {
    bar.reset();
    bar.set_length(resources.len() as u64);
    bar.set_message(format!("Ordering {resource_type} resources"));

    let mut errors_writer = ErrorsWriter::from_ctx(ctx, pkg_name)?;

    let resources = sort_resources_by_dependencies(resource_type, resources);

    bar.set_message(format!("Checking {resource_type} resources"));

    let requests = resources.into_iter().map(|resource| async {
        let exists = find_installed_resource(ctx.fhir_client, &resource.info).await?;

        bar.inc(1);

        anyhow::Ok((resource, exists))
    });

    let result = stream::iter(requests)
        .buffered(ctx.parallel_search_requests)
        .try_collect::<Vec<_>>()
        .await;

    let resources = match result {
        Ok(resources) => resources,
        Err(err) => {
            eprintln!("{err:#}");
            return Ok(0);
        }
    };

    bar.reset();

    let mut processed_count = 0;

    for (resource, existing) in resources {
        bar.set_message(format!(
            "{} {resource_type} {}",
            ctx.action.bar_prefix(),
            resource.info.id
        ));

        match ctx.action {
            Action::Install => {
                let source_path = resource.source_path.clone();
                let current_index = PackageIndex::default();
                match process_resource(
                    ctx,
                    resource_type,
                    resource,
                    &current_index,
                    pkg_name,
                    bar,
                    existing,
                )
                .await
                {
                    Ok(result) => {
                        processed_count += 1;
                        current_progress
                            .lock()
                            .unwrap()
                            .report
                            .add_install_result(result);
                    }
                    Err(err) => {
                        log_resource_error(
                            &mut errors_writer,
                            err,
                            &source_path,
                            pkg_name,
                            bar,
                            None,
                        );
                        current_progress.lock().unwrap().report.errors += 1;
                    }
                }
            }
            Action::Uninstall => match existing {
                Some(existing) => match ctx.fhir_client.delete(resource_type, &existing.id).await {
                    Ok(()) => {
                        bar.suspend(|| {
                            println!(
                                "Deleted {resource_type} {}",
                                style(resource.info.canonical_url().unwrap_or(existing.id)).bold()
                            );
                        });
                        processed_count += 1;
                        current_progress.lock().unwrap().report.removed += 1;
                    }
                    Err(err) => {
                        log_resource_error(
                            &mut errors_writer,
                            err,
                            Path::new(&format!("{resource_type}/{}", existing.id)),
                            pkg_name,
                            bar,
                            Some(&ResourceInfo {
                                resource_type: resource_type.to_owned(),
                                id: existing.id,
                                url: None,
                                version: None,
                            }),
                        );

                        current_progress.lock().unwrap().report.errors += 1;
                    }
                },
                None => {
                    processed_count += 1;
                }
            },
            Action::Check => {
                match existing {
                    Some(_) => {
                        current_progress.lock().unwrap().report.already_existed += 1;
                    }
                    None => {
                        missing_resources.push(resource);
                    }
                }
                processed_count += 1;
            }
        }

        bar.inc(1);
    }

    Ok(processed_count)
}

pub async fn process_resource(
    ctx: &PackageContext<'_>,
    resource_type: &str,
    mut resource: Resource,
    current_index: &PackageIndex,
    current_package: &str,
    bar: &ProgressBar,
    existing_resource: Option<ExistingResource>,
) -> Result<InstallResult, FhirError> {
    if ctx.existing_resources_behaviour == ExistingResourceBehaviour::Skip
        && existing_resource.is_some()
    {
        return Ok(InstallResult::Skipped);
    }

    if !ctx.skip_preprocessing {
        preprocess_resource(
            &mut resource,
            ctx.fhir_client,
            ctx.skip_strict_reference_versions,
            resource_type,
            current_index,
            bar,
        )
        .await?;
    }

    let result = if let Some(existing_resource) = existing_resource {
        let ExistingResource {
            id,
            data: existing_data,
        } = existing_resource;

        resource.set_id(id);

        if ctx.existing_resources_behaviour == ExistingResourceBehaviour::Overwrite
            || is_resource_changed(existing_data, resource.data.clone())
        {
            ctx.fhir_client
                .upsert(resource_type, &resource.info.id, &resource.data)
                .await?;
            InstallResult::Updated
        } else {
            InstallResult::Skipped
        }
    } else {
        ctx.fhir_client
            .upsert(resource_type, &resource.info.id, &resource.data)
            .await?;

        InstallResult::Created
    };

    bar.suspend(|| {
        if result != InstallResult::Skipped {
            print!(
                "[{}] {result:?} {resource_type} ",
                style(current_package).bold()
            );
            if let Some(url) = &resource.info.url {
                println!("{}", style(url).bold());
            } else {
                println!("{}", style(resource.source_path.display()).bold());
            }
        }
    });

    Ok(result)
}

#[derive(Debug, PartialEq)]
pub enum InstallResult {
    Created,
    Updated,
    Skipped,
}

/// Returns true if the resource was altered in any way
pub async fn preprocess_resource(
    resource: &mut Resource,
    fhir_client: &FhirClient,
    skip_strict_reference_versions: bool,
    resource_type: &str,
    current_index: &PackageIndex,
    bar: &ProgressBar,
) -> Result<bool, FhirError> {
    let mut changed = false;

    if resource.data.get("url").is_some() {
        let id = Uuid::new_v4();
        resource.set_id(id.to_string());
        changed = true;
    }

    if resource_type == "StructureDefinition" {
        if resource.data.get("snapshot").is_none() {
            bar.suspend(|| {
                println!(
                    "{} {resource_type} {} is missing a snapshot, generating",
                    style("Note:").bold(),
                    style(resource.source_path.display()).bold()
                );
            });

            let mut snapshot_response = fhir_client.snapshot(&resource.data).await?;
            let snapshot = snapshot_response
                .as_object_mut()
                .and_then(|obj| obj.remove("snapshot"))
                .ok_or_else(|| {
                    FhirError::Other(anyhow!(
                        "Snapshot operation response does not have snapshot field"
                    ))
                })?;

            if let Some(obj) = resource.data.as_object_mut() {
                obj.insert("snapshot".to_owned(), snapshot);
                changed = true;
            }
        }

        if !skip_strict_reference_versions {
            let count = process_definition_references(&mut resource.data, current_index);
            if count > 0 {
                bar.suspend(|| {
                    println!("{} {count} profile reference fields were normalized to contain an explicit version in profile {}", style("Note:").bold(), style(resource.source_path.display()).bold());
                });
                changed = true;
            }
        }
    }

    Ok(changed)
}

/// Normalizes references within the current package to point to specific versions of profiles
fn process_definition_references(definition_data: &mut Value, current_index: &PackageIndex) -> u64 {
    let mut changed_count = 0;

    if let Some(snapshot) = definition_data.get_mut("snapshot") {
        changed_count += process_definition_snapshot_references(snapshot, current_index);
    }
    if let Some(differential) = definition_data.get_mut("differential") {
        changed_count += process_definition_snapshot_references(differential, current_index);
    }

    changed_count
}

/// Works for differential too
fn process_definition_snapshot_references(
    snapshot: &mut Value,
    current_index: &PackageIndex,
) -> u64 {
    let mut changed_count = 0;

    if let Some(Value::Array(elements)) = snapshot.get_mut("element") {
        for element_definition in elements {
            if let Some(Value::Array(element_types)) = element_definition.get_mut("type") {
                for element_type in element_types {
                    for field in ["profile", "targetProfile"] {
                        if let Some(Value::Array(profiles)) = element_type.get_mut(field) {
                            for profile in profiles {
                                if let Value::String(reference) = profile {
                                    if normalize_profile_reference(reference, current_index) {
                                        changed_count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    changed_count
}

fn normalize_profile_reference(reference: &mut String, current_index: &PackageIndex) -> bool {
    if reference.split_once('|').is_some() {
        // Reference is already versioned
        return false;
    }

    // Try to find the latest version of a profile with such URL in the current index
    // If it exists, use that version in the reference explicitly
    if let Some(max_version) = current_index
        .files
        .iter()
        .filter(|file| file.resource_info.url.as_ref() == Some(&*reference))
        .flat_map(|file| &file.resource_info.version)
        .max()
    {
        *reference = format!("{reference}|{max_version}");
    }

    true
}

pub async fn check_package_installed<'a>(
    package_index: &'a PackageIndex,
    client: &FhirClient,
    total_progress: &ProgressBar,
    parallel_search_requests: usize,
) -> anyhow::Result<PackageInstallStatus<'a>> {
    total_progress.reset();
    total_progress.set_length(package_index.files.len() as u64);
    total_progress.set_message("Checking resources");

    let requests = package_index.files.iter().map(|file| async move {
        let existing = find_installed_resource(client, &file.resource_info).await?;

        total_progress.inc(1);

        anyhow::Ok((file, existing))
    });

    let mut stream = stream::iter(requests).buffer_unordered(parallel_search_requests);

    let mut resources = Vec::with_capacity(package_index.files.len());

    while let Some((file, existing_resource)) = stream.try_next().await? {
        resources.push((file, existing_resource));
    }

    total_progress.reset();

    Ok(resources)
}

pub type PackageInstallStatus<'a> = Vec<(&'a PackageIndexFile, Option<ExistingResource>)>;

pub struct ExistingResource {
    pub id: String,
    pub data: Value,
}

/// Returns the ids of existing resources
pub(super) async fn find_installed_resource(
    client: &FhirClient,
    resource_info: &ResourceInfo,
) -> anyhow::Result<Option<ExistingResource>> {
    let mut search_params: Vec<(&str, &str)> = vec![];
    if let Some(url) = &resource_info.url {
        search_params.push(("url", url));

        if let Some(version) = &resource_info.version {
            search_params.push(("version", version));
        }
    } else {
        search_params.push(("_id", &resource_info.id));
    }

    let bundle = client
        .search::<Value>(&resource_info.resource_type, &search_params)
        .await
        .context("Could not search currently installed resources")?;

    let existing_resource = bundle
        .entry
        .into_iter()
        .filter_map(|entry| entry.resource)
        .find_map(|data| {
            let url = data.get("url").and_then(Value::as_str);
            let version = data.get("version").and_then(Value::as_str);
            let id = data
                .get("id")
                .and_then(Value::as_str)
                .expect("Server returned resource without a valid id")
                .to_owned();

            let matches = (url.is_some()
                && url == resource_info.url.as_deref()
                && version == resource_info.version.as_deref())
                || (resource_info.url.is_none() && id == resource_info.id);

            if matches {
                Some(ExistingResource { id, data })
            } else {
                None
            }
        });

    Ok(existing_resource)
}

#[cfg(test)]
mod tests {
    use crate::installer::{
        package::FhirPackage, processor::process_definition_references, resource::Resource,
    };

    #[tokio::test]
    async fn preproces_normalize_references() {
        let package = FhirPackage::new("./tests/hl7.fhir.us.core@4.0.0".into());
        let index = package.read_index().unwrap();

        let index_file = index
            .files
            .iter()
            .find(|file| file.filename == "StructureDefinition-pediatric-weight-for-height.json")
            .unwrap();

        let raw_data = std::fs::read_to_string(package.dir.join(index_file.get_path())).unwrap();
        let data = serde_json::from_str(&raw_data).unwrap();
        let mut resource = Resource {
            data,
            info: index_file.resource_info.clone(),
            source_path: index_file.get_path(),
        };

        process_definition_references(&mut resource.data, &index);

        let target_profile = &resource
            .data
            .pointer("/snapshot/element/27/type/0/targetProfile")
            .unwrap()
            .as_array()
            .unwrap()[0];
        assert_eq!(
            "http://hl7.org/fhir/us/core/StructureDefinition/us-core-patient|4.0.0",
            target_profile.as_str().unwrap(),
        );
    }
}
