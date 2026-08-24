---
description: How to publish FHIR conformance resources stored as loose files in a repository to your FHIR server.
---

# Install fixtures from a repository

Not every FHIR project is structured as a formal Implementation Guide package. It's common to maintain conformance resources — StructureDefinitions, SearchParameters, CodeSystems, ValueSets, and similar files — as plain JSON files organized into folders in a Git repository.

This guide explains how to publish those resources to a FHIR server using Kodjin CLI, whether you need to push one file, one folder, or the entire repository at once.

**What you need before starting:**

- Kodjin CLI installed and a default server configured (see [First-time setup](guide-first-time-setup.md))
- A local clone of your repository

---

## Example repository structure

The examples in this guide use the following folder layout:

```
my-fhir-repo/
├── definitions/
│   └── us-core/
│       ├── StructureDefinition-sdc-questionnaireresponse.json
│       └── ...
├── search/
│   └── base/
│       ├── activitydefinition-composed-of.json
│       └── ...
└── terminology/
    └── us-core/
        ├── code_system/
        │   ├── CodeSystem-2.16.840.1.113883.6.238.json
        │   └── ...
        └── value_set/
            ├── 2.16.840.1.113762.1.4.1099.53.json
            └── ...
```

All examples below assume your terminal is open at the root of this repository.

---

## Install a single file

Use `--type file` followed by the path to the file:

```shell
$ kodjin-cli install --type file definitions/us-core/StructureDefinition-sdc-questionnaireresponse.json
```

You can also install several specific files in one command by listing their paths separated by spaces:

```shell
$ kodjin-cli install --type file \
  definitions/us-core/StructureDefinition-sdc-questionnaireresponse.json \
  terminology/us-core/code_system/CodeSystem-2.16.840.1.113883.6.238.json
```

Use this when you've edited one or two files and want to push only those changes without touching the rest.

---

## Install a single folder

Use `--type directory` followed by the path to the folder. Kodjin CLI will recursively find and upload all FHIR resource files inside it.

Since your repository folders don't contain a formal `package.json`, add `--skip-dependencies` to prevent Kodjin CLI from trying to resolve package dependencies that don't apply here:

```shell
# Upload all StructureDefinitions for us-core
$ kodjin-cli install --type directory definitions/us-core --skip-dependencies

# Upload all base search parameters
$ kodjin-cli install --type directory search/base --skip-dependencies

# Upload all CodeSystems
$ kodjin-cli install --type directory terminology/us-core/code_system --skip-dependencies

# Upload all ValueSets
$ kodjin-cli install --type directory terminology/us-core/value_set --skip-dependencies
```

Use this when you've made changes in a specific area and want to push just that category of resources.

---

## Install multiple folders selectively

Run the command once per folder, choosing exactly which parts of the repository to publish:

```shell
# Publish definitions and terminology, but not search parameters
$ kodjin-cli install --type directory definitions/us-core --skip-dependencies
$ kodjin-cli install --type directory terminology/us-core/code_system --skip-dependencies
$ kodjin-cli install --type directory terminology/us-core/value_set --skip-dependencies
```

This gives you fine-grained control over what lands on the server.

---

## Install the entire repository at once

To publish everything in one command, run from the repository root:

```shell
$ kodjin-cli install --type directory . --skip-dependencies
```

The `.` refers to the current directory. Kodjin CLI will scan all subdirectories recursively and upload every FHIR resource file it finds.

---

## Update resources that are already on the server

By default, Kodjin CLI skips files that are already installed. If you've edited existing resources and want the server to reflect your changes, add `--existing-resources sync`:

```shell
# Sync only changed files in the definitions folder
$ kodjin-cli install --type directory definitions/us-core \
  --skip-dependencies \
  --existing-resources sync
```

Use `--existing-resources overwrite` to replace all resources unconditionally, regardless of whether they've changed.

---

## Check what's on the server before installing

The `check` command works with directories and files just like `install`, but makes no changes. Use it to see what would be uploaded before committing:

```shell
$ kodjin-cli check --type directory definitions/us-core --skip-dependencies
```

This reports which files are missing from the server and which are already present, so you know exactly what `install` will do.

---

## Remove resources from the server

To remove resources that were installed from a folder:

```shell
$ kodjin-cli uninstall --type directory definitions/us-core
```

This removes only the conformance resources — patient data and other non-conformance content on the server is not affected.
