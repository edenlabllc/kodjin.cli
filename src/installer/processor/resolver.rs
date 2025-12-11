use crate::installer::resource::Resource;
use indexmap::{IndexMap, IndexSet};
use serde_json::Value;

struct ItemInfo<'a> {
    index: usize,
    dependencies: IndexSet<&'a str>,
}

/// Returns a list of indexes according to the processing order
pub fn sort_resources_by_dependencies(
    resource_type: &str,
    mut resources: Vec<Resource>,
) -> Vec<Resource> {
    resources.sort_by_key(|resource| resource.info.id.clone());

    let order = get_resources_order(resource_type, &resources);
    sort_with_order(resources, &order)
}

pub fn get_resources_order(resource_type: &str, resources: &[Resource]) -> Vec<usize> {
    match resource_type {
        "StructureDefinition" => {
            let mut urls: IndexMap<&str, Vec<usize>> = IndexMap::with_capacity(resources.len());

            for (i, resource) in resources.iter().enumerate() {
                if let Some(url) = &resource.info.url {
                    urls.entry(url).or_default().push(i);
                }
            }

            let mut item_info: IndexMap<&str, Vec<ItemInfo<'_>>> = urls
                .iter()
                .map(|(&url, indexes)| {
                    let info = indexes
                        .iter()
                        .map(|&index| {
                            let resource = &resources[index];

                            let mut dependencies = get_dependencies(&resource.data, resource_type);
                            dependencies.retain(|url| urls.contains_key(url));

                            ItemInfo {
                                index,
                                dependencies,
                            }
                        })
                        .collect();

                    (url, info)
                })
                .collect();

            let mut order = Vec::with_capacity(item_info.len());
            while let Some((_, items)) = item_info.pop() {
                for current_item in items {
                    collect_item_indexes(&mut item_info, current_item, &mut order);
                }
            }
            order
        }
        _ => (0..resources.len()).collect(),
    }
}

fn sort_with_order(resources: Vec<Resource>, order: &[usize]) -> Vec<Resource> {
    let mut resources_optional = resources.into_iter().map(Some).collect::<Vec<_>>();

    order
        .iter()
        .map(|&i| resources_optional[i].take().unwrap())
        .collect()
}

fn collect_item_indexes(
    items: &mut IndexMap<&str, Vec<ItemInfo<'_>>>,
    current_item: ItemInfo<'_>,
    order: &mut Vec<usize>,
) {
    for dependency in current_item.dependencies {
        if let Some(dependency_items) = items.swap_remove(dependency) {
            for item in dependency_items {
                collect_item_indexes(items, item, order);
            }
        }
    }

    order.push(current_item.index);
}

fn get_dependencies<'a>(data: &'a Value, resource_type: &str) -> IndexSet<&'a str> {
    let mut urls = IndexSet::new();

    if resource_type == "StructureDefinition" {
        get_extension_urls(data, &mut urls)
    }

    urls
}

fn get_extension_urls<'a>(data: &'a Value, output: &mut IndexSet<&'a str>) {
    match data {
        Value::Object(map) => {
            if let Some(extension) = map.get("extension") {
                match extension {
                    Value::Object(extension) => {
                        if let Some(Value::String(url)) = extension.get("url") {
                            output.insert(url.as_str());
                        }
                    }
                    Value::Array(items) => {
                        for extension in items {
                            if let Some(Value::String(url)) = extension.get("url") {
                                output.insert(url.as_str());
                            }
                        }
                    }
                    _ => (),
                }
            }

            for value in map.values() {
                get_extension_urls(value, output);
            }
        }
        Value::Array(values) => {
            for value in values {
                get_extension_urls(value, output);
            }
        }
        _ => (),
    }
}

#[cfg(test)]
mod tests {
    use super::get_dependencies;
    use crate::installer::{
        load_file, processor::resolver::sort_resources_by_dependencies, resource::Resource,
    };
    use indexmap::{IndexMap, IndexSet};
    use pretty_assertions::assert_eq;
    use serde_json::Value;
    use std::fs;

    #[test]
    fn observation_occupation_required_definitions() {
        let data = include_str!("../../../tests/us-core-v6-subset/StructureDefinition-us-core-observation-occupation.json");
        let data: Value = serde_json::from_str(data).unwrap();

        let required_urls = get_dependencies(&data, "StructureDefinition");
        let expected_urls = IndexSet::from([
            "http://hl7.org/fhir/StructureDefinition/structuredefinition-standards-status",
            "http://hl7.org/fhir/StructureDefinition/elementdefinition-isCommonBinding",
            "http://hl7.org/fhir/StructureDefinition/elementdefinition-bindingName",
            "http://hl7.org/fhir/StructureDefinition/elementdefinition-bestpractice",
            "http://hl7.org/fhir/StructureDefinition/structuredefinition-fhir-type",
            "http://hl7.org/fhir/StructureDefinition/structuredefinition-display-hint",
            "http://hl7.org/fhir/StructureDefinition/elementdefinition-bestpractice-explanation",
            "http://hl7.org/fhir/us/core/StructureDefinition/uscdi-requirement",
            "http://hl7.org/fhir/StructureDefinition/elementdefinition-maxValueSet",
        ]);

        assert_eq!(expected_urls, required_urls);
    }

    fn assert_order(resources: Vec<Resource>, expected_order: &[&str]) {
        let resource_ids = resources
            .iter()
            .map(|resource| resource.info.id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(expected_order, &resource_ids);
    }

    fn read_resources(dir: &str) -> IndexMap<String, Vec<Resource>> {
        let dir = fs::read_dir(dir).unwrap();

        let mut resources = IndexMap::new();

        for item in dir {
            let item = item.unwrap();
            let path = item.path();
            load_file(&mut resources, &path, &path).unwrap();
        }

        resources
    }

    #[test]
    fn us_core_subset_sort() {
        let mut resources = read_resources("tests/us-core-v6-subset");

        let definitions = resources.swap_remove("StructureDefinition").unwrap();
        let sorted = sort_resources_by_dependencies("StructureDefinition", definitions);

        assert_order(
            sorted,
            &[
                "610-uscdi-requirement",
                "610-us-core-observation-occupation",
                "610-head-occipital-frontal-circumference-percentile",
            ],
        );
    }
}
