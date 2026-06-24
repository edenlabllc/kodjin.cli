---
description: How to find and inspect a FHIR package before installing it.
---

# Look up a package before installing

Before installing an Implementation Guide, it's worth checking what it contains and what other packages it depends on. This helps you avoid surprises — like an unexpectedly large install or a version that doesn't match your server's FHIR version.

**What you need before starting:**

- Kodjin CLI installed and a default server configured (see [First-time setup](guide-first-time-setup.md))
- The name of the package you want to look up

> **Where do I find package names?**  
> Browse [packages.simplifier.net](https://packages.simplifier.net) — the official FHIR package registry. Package names follow the pattern `publisher.fhir.realm.name`, for example `hl7.fhir.us.core`. Always include a version number after `@` (e.g., `@4.0.0`) to get a specific release.

---

## View package details

The `info` command fetches metadata for a package directly from the registry:

```shell
$ kodjin-cli info hl7.fhir.us.core@4.0.0

Name: hl7.fhir.us.core
Version: 4.0.0
Author: HL7 International - US Realm Steering Committee
Description: The US Core Implementation Guide is based on FHIR Version R4 and defines the minimum conformance requirements for accessing patient data. The Argonaut pilot implementations, ONC 2015 Edition Common Clinical Data Set (CCDS), and ONC U.S. Core Data for Interoperability (USCDI) v1 provided the requirements for this guide. The prior Argonaut search and vocabulary requirements, based on FHIR DSTU2, are updated in this guide to support FHIR Version R4. This guide was used as the basis for further testing and guidance by the Argonaut Project Team to provide additional content and guidance specific to Data Query Access for purpose of ONC Certification testing. These profiles are the foundation for future US Realm FHIR implementation guides. In addition to Argonaut, they are used by DAF-Research, QI-Core, and CIMI. Under the guidance of HL7 and the HL7 US Realm Steering Committee, the content will expand in future versions to meet the needs specific to the US Realm.
These requirements were originally developed, balloted, and published in FHIR DSTU2 as part of the Office of the National Coordinator for Health Information Technology (ONC) sponsored Data Access Framework (DAF) project. For more information on how DAF became US Core see the US Core change notes. (built Mon, Jun 28, 2021 19:09+0000+00:00)
FHIR Versions: 4.0.1
Dependencies: hl7.fhir.r4.core@4.0.1, hl7.fhir.uv.bulkdata@1.0.1, us.nlm.vsac@0.3.0
Contents: CapabilityStatement: 2, CodeSystem: 5, ImplementationGuide: 2, OperationDefinition: 1, SearchParameter: 74, StructureDefinition: 39, ValueSet: 25
```

What the output tells you:

- **FHIR Versions** — The FHIR version this package requires. Make sure it matches your server (check with `kodjin-cli metadata`).
- **Dependencies** — Other packages this one relies on. Kodjin CLI will install these automatically, but it's good to know they exist.
- **Contents** — The types and counts of conformance resources inside the package.

---

## View the full dependency tree

Dependencies can have their own dependencies. The `tree` command shows the entire chain:

```shell
$ kodjin-cli tree hl7.fhir.us.core@4.0.0

 - hl7.fhir.us.core@4.0.0 (148 resources)
   - hl7.fhir.uv.bulkdata@1.0.1 (6 resources)
     - hl7.fhir.r4.core@4.0.1 (4581 resources)
   - us.nlm.vsac@0.3.0 (8916 resources)
   - hl7.fhir.r4.core@4.0.1 (4581 resources)
```

Reading this output:

- Each line is a package. Indentation shows nesting — a package listed under another is its dependency.
- The number in parentheses is the resource count for that package.
- `hl7.fhir.r4.core@4.0.1` is the base FHIR R4 specification package — it contains thousands of resources and should already be present on most FHIR servers, so Kodjin CLI will skip it.

This is useful for estimating the total size of an install before you commit to it. Also you can decide to skip dependencies with `--skip-dependencies`

---

## What's next

Once you know what the package contains, you're ready to install it: [Install an Implementation Guide](guide-install-an-IG.md)
