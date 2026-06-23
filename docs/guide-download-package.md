---
description: How to download a FHIR package to disk for inspection or local modification.
---

# Download a package for local inspection

The `download` command saves a package from the FHIR registry to your computer without installing anything on the server. This is useful when you want to inspect the resource files, understand what a package contains, or make modifications before installing.

The files are saved to your current directory.

---

## Download the raw package

```shell
$ kodjin-cli download hl7.fhir.us.core@4.0.0
Package downloaded to ./hl7.fhir.us.core@4.0.0
```

After this command, you'll find a folder named `hl7.fhir.us.core@4.0.0` in your current directory. Inside it are all the raw resource files exactly as they were published — JSON files for StructureDefinitions, ValueSets, CodeSystems, and so on.

You can open and edit these files before installing them locally with `kodjin-cli install --type directory`.

---

## Download with preprocessing applied

When Kodjin CLI installs a package, it automatically preprocesses the resources before uploading them:

- Generates new IDs for canonical resources
- Generates missing snapshots for StructureDefinition resources
- Rewrites profile references to be version-specific

If you want the downloaded files to match exactly what would be uploaded by `install`, add the `--preprocess` flag:

```shell
$ kodjin-cli download --preprocess hl7.fhir.us.core@4.0.0
Preprocessed file package/SearchParameter-us-core-immunization-patient.json
Note: 25 profile reference fields were normalized to contain an explicit version in profile package/StructureDefinition-us-core-diagnosticreport-lab.json
Preprocessed file package/StructureDefinition-us-core-diagnosticreport-lab.json
...
Package downloaded to ./hl7.fhir.us.core@4.0.0
```

The output lists each file that was processed and notes what changed. This is useful if you want to review the exact resources that would land on the server before committing to an install.

---

## Typical workflow: download, review, install

```shell
# 1. Download the package with preprocessing
$ kodjin-cli download --preprocess hl7.fhir.us.core@4.0.0

# 2. Inspect the files (open the folder in your editor or file browser)

# 3. Once satisfied, install from the local folder
$ kodjin-cli install --type directory ./hl7.fhir.us.core@4.0.0
```

This gives you full visibility into what gets installed on the server before anything is uploaded.

---

## What's next

- To know how to install from the local directory, see [Install from a local directory](./guide-install-from-local-dir.md)
