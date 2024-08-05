use crate::client::FhirClient;
use anyhow::Context;
use console::style;
use indicatif::{HumanDuration, ProgressBar};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

const RESOURCE_TYPES_ORDER: &[&str] = &[
    "StructureDefinition",
    "SearchParameter",
    "CodeSystem",
    "ValueSet",
    "ConceptMap",
];

struct Resource {
    data: Value,
    id: String,
    /// Relative path where the resource was loaded from
    source_path: PathBuf,
}

pub fn process_directory(path: &Path, client: &FhirClient) -> anyhow::Result<()> {
    let started_at = Instant::now();

    let bar = ProgressBar::new_spinner().with_message("Loading data");

    // Grouped by resource type
    let mut resources: HashMap<String, Vec<Resource>> = HashMap::new();

    let paths = load_file_list(path)?;
    for file_path in paths {
        let relative_path = file_path.strip_prefix(path)?;
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

    let processed_count = process_resources(resources, client);

    println!(
        "Successfully processed {} resources in {}",
        style(processed_count).bold(),
        style(HumanDuration(started_at.elapsed())).bold()
    );

    Ok(())
}

fn process_resources(mut resources: HashMap<String, Vec<Resource>>, client: &FhirClient) -> usize {
    let bar = ProgressBar::new_spinner().with_message("Uploading resources");

    let mut processed_count = 0;

    // First we process resources in the defined order
    for resource_type in RESOURCE_TYPES_ORDER {
        if let Some(resources) = resources.remove(*resource_type) {
            processed_count += process_resources_type(resource_type, resources, client, &bar);
        }
    }

    // Process remaining resource types which were not in the list
    for (resource_type, resources) in resources.into_iter() {
        processed_count += process_resources_type(&resource_type, resources, client, &bar);
    }

    bar.finish_and_clear();

    processed_count
}

/// Returns the number of resources which were successfully uploaded
fn process_resources_type(
    resource_type: &str,
    resources: Vec<Resource>,
    client: &FhirClient,
    bar: &ProgressBar,
) -> usize {
    let mut count = 0;

    for resource in resources {
        bar.tick();
        bar.set_message(format!("Uploading {resource_type} {}", resource.id));

        match process_resource(resource_type, &resource, client) {
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

    count
}

fn process_resource(
    resource_type: &str,
    resource: &Resource,
    client: &FhirClient,
) -> anyhow::Result<()> {
    let payload = serde_json::to_string(&resource.data)?;
    client.upsert(resource_type, &resource.id, &payload)?;
    Ok(())
}

fn load_file(
    resources: &mut HashMap<String, Vec<Resource>>,
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
