use std::time::Duration;

use super::{Action, InstallContext, Resource};
use crate::client::FhirClient;
use console::style;
use indexmap::IndexMap;
use indicatif::{ProgressBar, ProgressStyle};

const RESOURCE_TYPES_ORDER: &[&str] = &[
    "StructureDefinition",
    "SearchParameter",
    "CodeSystem",
    "ValueSet",
    "ConceptMap",
];

pub async fn process_resources(
    ctx: &InstallContext,
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
    ctx: &InstallContext,
    resource_type: &str,
    resources: Vec<Resource>,
    bar: &ProgressBar,
) -> usize {
    let mut count = 0;

    for resource in resources {
        bar.set_message(format!(
            "{} {resource_type} {}",
            ctx.action.bar_prefix(),
            resource.id
        ));

        match ctx.action {
            Action::Install => {
                match process_resource(resource_type, &resource, &ctx.client).await {
                    Ok(()) => count += 1,
                    Err(err) => {
                        bar.suspend(|| {
                            let msg = format!(
                                "Warning: could not process file {:?}: {err:#}",
                                resource.source_path
                            );
                            println!("{}", style(msg).yellow())
                        });
                    }
                }
            }
            Action::Uninstall => match ctx.client.delete(resource_type, &resource.id).await {
                Ok(()) => {
                    count += 1;
                }
                Err(err) => {
                    bar.suspend(|| {
                        let msg = format!(
                            "Warning: could not delete resource {resource_type}/{}: {err:#}",
                            resource.id
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

async fn process_resource(
    resource_type: &str,
    resource: &Resource,
    client: &FhirClient,
) -> anyhow::Result<()> {
    let payload = serde_json::to_string(&resource.data)?;
    client.upsert(resource_type, &resource.id, &payload).await?;
    Ok(())
}
