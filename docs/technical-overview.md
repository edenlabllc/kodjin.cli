---
description: Technical overview of Kodjin CLI — architecture, package resolution, resource preprocessing, and server management.
---

## What is Kodjin CLI?

Kodjin CLI is a command-line tool written in Rust for managing FHIR Implementation Guides (IGs) on FHIR R4 servers. It resolves and installs IG packages from the [FHIR package registry](https://packages.simplifier.net) (or a custom registry), handles transitive dependency resolution, and uploads conformance resources to a target FHIR server via its REST API.

## Package resolution and dependency handling

FHIR IGs are distributed as `.tgz` packages following the [FHIR package specification](http://hl7.org/fhir/packages.html). Each package declares its dependencies in `package.json`. Kodjin CLI resolves the full dependency graph recursively before installation - equivalent to running `kodjin-cli tree` - and installs missing packages in dependency order.

```shell
$ kodjin-cli tree hl7.fhir.us.core@4.0.0
 - hl7.fhir.us.core@4.0.0 (148 resources)
   - hl7.fhir.uv.bulkdata@1.0.1 (6 resources)
     - hl7.fhir.r4.core@4.0.1 (4581 resources)
   - hl7.fhir.r4.core@4.0.1 (4581 resources)
   - us.nlm.vsac@0.3.0 (8916 resources)
```

The `--skip-dependencies` flag bypasses this step. Use with caution — missing dependencies will likely cause validation failures on the server.

## Resource preprocessing

Before uploading, Kodjin CLI preprocesses resources by default:

- **ID generation** — Canonical resources (those with both `url` and `version`) get deterministic new IDs to avoid conflicts across packages.
- **Snapshot generation** — `StructureDefinition` resources missing a `snapshot` element have one generated automatically.
- **Version-specific references** — Profile references within a package are rewritten to include explicit version suffixes (e.g., `|4.0.0`), ensuring cross-package references resolve to the correct version.

All three steps can be skipped individually or together with `--skip-preprocessing` or `--skip-strict-reference-versions`.

## Conflict resolution

When installing into a server that already has resources loaded, the `--existing-resources` flag controls behaviour:

| Value              | Behaviour                                                  |
| ------------------ | ---------------------------------------------------------- |
| `skip` *(default)* | Leaves existing resources untouched                        |
| `sync`             | Updates resources only if content differs from the package |
| `overwrite`        | Unconditionally replaces existing resources                |

## Installation sources

Kodjin CLI supports three source types via `--type`:

- `package` — Fetches from the FHIR registry by name and version (e.g., `hl7.fhir.us.core@4.0.0`)
- `directory` — Installs from a local directory containing a valid FHIR package structure
- `file` — Installs specific resource files without the full package context

## Server management

Servers are stored in local config and referenced by URL or name. The active default server is used by all commands unless overridden at runtime. Multiple environments (dev, staging, prod) can be registered and switched between without changing command syntax.

```shell
$ kodjin-cli server add --name PROD https://fhir.example.com/r4
$ kodjin-cli server default PROD
$ kodjin-cli install hl7.fhir.us.core@4.0.0
```

## Performance

Parallel server requests during install and check operations are configurable via `--parallel-search-requests` (default: 10). Increasing this can reduce wall-clock time for large packages but may hit server rate limits or increase load.

## Where to go next

- [Installation](installation.md) — Get Kodjin CLI installed on your machine
- [First-time setup guide](./guide-first-time-setup.md) — Examples of use cases
