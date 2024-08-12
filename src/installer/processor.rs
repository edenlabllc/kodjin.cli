use super::{
    package::{FhirPackage, PackageIndexFile},
    resource::{Resource, ResourceInfo},
    Action, InstallContext,
};
use crate::client::FhirClient;
use console::style;
use futures::{stream, StreamExt, TryStreamExt};
use indexmap::IndexMap;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use uuid::Uuid;

const CONCURRENT_SEARCH_REQUESTS: usize = 20;
const RESOURCE_TYPES_ORDER: &[&str] = &[
    "StructureDefinition",
    "SearchParameter",
    "CodeSystem",
    "ValueSet",
    "ConceptMap",
];

pub async fn process_resources(
    ctx: InstallContext<'_>,
    mut resources: IndexMap<String, Vec<Resource>>,
) -> usize {
    let count: usize = resources.values().map(|resources| resources.len()).sum();

    let bar = ProgressBar::new(count as u64)
        .with_message(format!("{} resources", ctx.action.bar_prefix()));
    bar.enable_steady_tick(Duration::from_millis(100));
    bar.set_style(ProgressStyle::with_template("{spinner} [{pos}/{len}] {msg}").unwrap());

    let mut processed_count = 0;

    // First we process resources in the defined order
    for resource_type in RESOURCE_TYPES_ORDER {
        if let Some(resources) = resources.shift_remove(*resource_type) {
            processed_count += process_resources_type(ctx, resource_type, resources, &bar).await;
        }
    }

    // Process remaining resource types which were not in the list
    for (resource_type, resources) in resources.into_iter() {
        processed_count += process_resources_type(ctx, &resource_type, resources, &bar).await;
    }

    bar.finish_and_clear();

    processed_count
}

/// Returns the number of resources which were successfully uploaded
async fn process_resources_type(
    ctx: InstallContext<'_>,
    resource_type: &str,
    resources: Vec<Resource>,
    bar: &ProgressBar,
) -> usize {
    let mut count = 0;

    for resource in resources {
        bar.set_message(format!(
            "{} {resource_type} {}",
            ctx.action.bar_prefix(),
            resource.info.id
        ));

        match ctx.action {
            Action::Install => {
                let source_path = resource.source_path.clone();
                match process_resource(resource_type, resource, ctx.fhir_client, bar).await {
                    Ok(()) => count += 1,
                    Err(err) => {
                        bar.suspend(|| {
                            let msg = format!(
                                "Warning: could not process file {source_path:?}: {err:#}",
                            );
                            println!("{}", style(msg).yellow())
                        });
                    }
                }
            }
            Action::Uninstall => match ctx
                .fhir_client
                .delete(resource_type, &resource.info.id)
                .await
            {
                Ok(()) => {
                    count += 1;
                }
                Err(err) => {
                    bar.suspend(|| {
                        let msg = format!(
                            "Warning: could not delete resource {resource_type}/{}: {err:#}",
                            resource.info.id
                        );
                        println!("{}", style(msg).yellow())
                    });
                }
            },
        }

        bar.inc(1);
    }

    count
}

pub async fn process_resource(
    resource_type: &str,
    mut resource: Resource,
    client: &FhirClient,
    _bar: &ProgressBar,
) -> anyhow::Result<()> {
    let exists = check_resource_installed(client, &resource.info).await?;
    if !exists {
        let id = Uuid::new_v4();
        resource.set_id(id.to_string());

        let payload = serde_json::to_string(&resource.data)?;
        client
            .upsert(resource_type, &resource.info.id, &payload)
            .await?;
    }

    Ok(())
}

/// Strip the resource of server-defined values such a `lastUpdated` and `versionId`
// fn strip_resource(data: &mut Value) {
//     if let Some(Value::Object(meta)) = data.get_mut("meta") {
//         meta.remove("lastUpdated");
//         meta.remove("versionId");

//         if meta.is_empty() {
//             data.as_object_mut().unwrap().remove("meta");
//         }
//     }
// }

pub async fn check_package_installed(
    package: &FhirPackage,
    client: &FhirClient,
    // progress: &MultiProgress,
    total_progress: &ProgressBar,
) -> anyhow::Result<PackageInstallStatus> {
    let index = package.read_index()?;

    total_progress.set_length(index.files.len() as u64);
    total_progress.set_message("Checking resources");
    // total_progress.set_style(
    //     ProgressStyle::with_template(&format!("{spinner} {}: [{pos}/{len}] {msg} [{wide_bar}]")
    //         .unwrap()
    //         .progress_chars("#>-"),
    // );
    // let total_progress = progress
    // // .add(
    //     ProgressBar::new(index.files.len() as u64).with_style(
    //         ProgressStyle::with_template("{spinner} [{pos}/{len}] {msg} [{wide_bar}]")
    //             .unwrap()
    //             .progress_chars("#>-"),
    //     ),
    // )
    // .with_message("Checking resources");

    let requests = index.files.into_iter().map(|file| async {
        // let bar = progress.add(
        //     ProgressBar::new_spinner()
        //         .with_message(format!("Checking {}", style(&file.filename).bold())),
        // );

        // bar.enable_steady_tick(Duration::from_millis(100));

        let exists = check_resource_installed(client, &file.resource_info).await?;

        // bar.finish_and_clear();

        total_progress.inc(1);

        anyhow::Ok((file, exists))
    });

    let missing = stream::iter(requests)
        .buffer_unordered(CONCURRENT_SEARCH_REQUESTS)
        .try_filter(|(_, exists)| {
            let exists = *exists;
            async move { !exists }
        })
        .map_ok(|(file, _)| file)
        .try_collect::<Vec<_>>()
        .await?;

    total_progress.reset();

    if missing.is_empty() {
        Ok(PackageInstallStatus::Installed)
    } else {
        Ok(PackageInstallStatus::NotInstalled(missing))
    }
}

pub enum PackageInstallStatus {
    Installed,
    NotInstalled(Vec<PackageIndexFile>),
}

async fn check_resource_installed(
    client: &FhirClient,
    resource_info: &ResourceInfo,
) -> anyhow::Result<bool> {
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
        .search::<ResourceInfo>(&resource_info.resource_type, &search_params)
        .await?;

    let exists = bundle.entry.iter().any(|entry| {
        entry.resource.as_ref().is_some_and(|resource| {
            (resource.url.is_some()
                && resource.url == resource_info.url
                && resource.version == resource_info.version)
                || (resource_info.url.is_none() && resource.id == resource_info.id)
        })
    });

    Ok(exists)
}
