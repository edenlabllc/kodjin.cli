---
description: How to install a FHIR package from a local directory or file instead of the registry.
---

# Install from a local directory

By default, Kodjin CLI fetches packages from the FHIR package registry. But sometimes you need to install from a local source — for example, when working with a custom package, a modified version of an existing guide, or a package that hasn't been published yet.

**What you need before starting:**

- A local directory containing a valid FHIR package (a folder with a `package.json` and FHIR resource files)
- An IG could be downloaded previously (see [Download a package for local inspection](./guide-download-package.md))
- A default server configured (see [First-time setup](guide-first-time-setup.md))

---

## Install from a directory

Use `--type directory` and provide the path to the package folder:

```shell
$ kodjin-cli install --type directory /path/to/my-package
```

To install from the current directory (useful if your terminal is already inside the package folder):

```shell
$ kodjin-cli install --type directory .
```

The `.` means "current directory." Kodjin CLI will read the package structure from there and upload its resources to the server.

---

## Install specific files

If you only need to upload one or a few resource files — rather than an entire package — use `--type file`:

```shell
$ kodjin-cli install --type file StructureDefinition-Patient.json ValueSet-Gender.json
```

List all the filenames you want separated by spaces. This skips dependency resolution and installs only the files you specify.

This is useful when:

- You've made a small edit to one resource and want to push just that change
- You're testing a single new resource before publishing the full package

---

## Notes on local packages

- Dependency resolution still applies unless you add `--skip-dependencies`. If your local package declares dependencies, Kodjin CLI will attempt to fetch them from the registry.
- To avoid fetching dependencies from the registry, combine `--type directory` with `--skip-dependencies`:

```shell
$ kodjin-cli install --type directory . --skip-dependencies
```

---

## What's next

If you want to download a registry package to disk first (to inspect or modify it before installing), see [Download a package for local inspection](guide-download-package.md).
