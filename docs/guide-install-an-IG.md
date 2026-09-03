---
description: How to install a FHIR Implementation Guide and verify what was installed.
---

# Install an Implementation Guide

Installing a package loads its conformance resources — StructureDefinitions, ValueSets, CodeSystems, SearchParameters, and others — onto your FHIR server. Once installed, the server can validate data against those profiles and support the workflows the Implementation Guide defines.

**What you need before starting:**

- Kodjin CLI installed and a default server configured (see [First-time setup](guide-first-time-setup.md))
- The package name and version you want to install (see [Look up a package](guide-look-up-a-package.md))

---

## Install a package

```shell
$ kodjin-cli install hl7.fhir.us.core@4.0.0
```

That's the complete command. Kodjin CLI will:

1. Fetch the package from the FHIR registry
2. Resolve all dependencies (other packages this one requires)
3. For each package, check which resources are already on the server
4. Upload only the missing resources

You'll see progress output as each package is processed. When it finishes, all 148 resources from `hl7.fhir.us.core@4.0.0` — plus its dependencies — will be on your server.

> **Already have some resources installed?** No problem. Kodjin CLI skips resources that are already present by default, so re-running `install` is safe. See [Update a package](guide-update-a-package.md) if you want to refresh existing resources.

---

## Check what's installed

Before installing, or after a partial install, you can ask Kodjin CLI to compare a package against the server and report what's missing — without making any changes.

```shell
$ kodjin-cli check hl7.fhir.us.core@4.0.0

The following files are missing:
ValueSet:
  - ValueSet-us-core-vital-signs.json (http://hl7.org/fhir/us/core/ValueSet/us-core-vital-signs|4.0.0)
SearchParameter:
  - SearchParameter-us-core-condition-code.json (http://hl7.org/fhir/us/core/SearchParameter/us-core-condition-code|4.0.0)
CodeSystem:
  - CodeSystem-condition-category.json (http://hl7.org/fhir/us/core/CodeSystem/condition-category|4.0.0)
Package hl7.fhir.us.core@4.0.0 is partially installed (39/148 resources present)
```

This output means 39 out of 148 resources are already on the server, and 3 specific resources are missing. No data was changed — `check` is read-only.

If you see `Package hl7.fhir.us.core@4.0.0 is fully installed`, everything is in place and no further action is needed.

To install whatever is missing, just run `install` after `check`:

```shell
$ kodjin-cli install hl7.fhir.us.core@4.0.0
```

---

## What's next

- The package is now installed and your server can use it for validation.
- If you need to update resources that were already installed, see [Update a package](guide-update-a-package.md).
- If something went wrong and you want to inspect the errors, see [Save error logs](guide-save-error.md).
