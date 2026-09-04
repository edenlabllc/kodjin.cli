---
description: Answers to common questions from new Kodjin CLI users.
---

# FAQ

## Installation & setup

**How do I install Kodjin CLI?**

Run the following command on Linux or macOS:

```shell
curl -fsSL https://edenlabllc-kodjin-cli.s3.eu-north-1.amazonaws.com/kodjin-cli/installer.sh | bash -
```

After installation, verify it works with:

```shell
kodjin-cli --version
```

---

**How do I update Kodjin CLI?**

Run the built-in update command:

```shell
kodjin-cli update
```

---

**Where does Kodjin CLI store its configuration?**

Configuration (registered servers, default server) is stored in the platform's standard application data directory:

| Platform | Location                             |
| -------- | ------------------------------------ |
| Linux    | `$XDG_DATA_HOME` or `~/.local/share` |
| macOS    | `~/Library/Application Support`      |
| Windows  | `%LOCALAPPDATA%`                     |

---

## Servers

**How do I know which server is currently set as default?**

```shell
$ kodjin-cli server list
```

The default server is marked `(default)` in the output.

---

**Can I use multiple servers?**

Yes. Register as many as you need with `server add` and switch between them using `--server` at runtime, or change the default with `server default`. See [command-server](command-server.md) for details.

---

**My server uses a self-signed TLS certificate. How do I connect?**

Use the `--insecure-certificates` flag to skip certificate validation. Only use this in trusted development or testing environments:

```shell
$ kodjin-cli --insecure-certificates install hl7.fhir.us.core@4.0.0
```

---

## Packages & installation

**How do I find the right package name?**

Browse the FHIR package registry at [packages.simplifier.net](https://packages.simplifier.net). Package names follow the format `publisher.fhir.realm.name`, for example `hl7.fhir.us.core`. Always specify a version with `@version` (e.g., `hl7.fhir.us.core@4.0.0`) to get a reproducible install.

---

**Do I need to install dependencies manually?**

No. Kodjin CLI resolves and installs all transitive dependencies automatically. You can preview them first with `kodjin-cli tree <package>`. If you want to skip dependency installation, use `--skip-dependencies`, but be aware that missing dependencies are likely to cause validation errors on the server.

---

**What happens if a resource is already installed on the server?**

By default, Kodjin CLI skips resources that already exist (`--existing-resources skip`). To update them, use:

- `--existing-resources sync` — updates only resources whose content differs from the package
- `--existing-resources overwrite` — replaces all resources unconditionally

Use `overwrite` with care in production environments.

---

**What's the difference between `check` and `install`?**

`check` compares a package against the server and reports what's missing — it makes no changes. `install` does the same comparison and then uploads the missing resources. Running `check` first is a good way to audit the current state before committing to an install.

---

**Can I install multiple packages at once?**

No, you can pass only one package name to `install` in a command:

```shell
$ kodjin-cli install hl7.fhir.us.core@4.0.0
```

---

**Can I install from a local package instead of the registry?**

Yes, using `--type directory`:

```shell
$ kodjin-cli install --type directory /path/to/my-package
```

This is useful for custom or modified packages not published to a registry.

---

## Preprocessing

**What does preprocessing do, and should I skip it?**

During install, Kodjin CLI automatically:

- Generates new deterministic IDs for canonical resources (those with `url` + `version`) to avoid ID collisions
- Generates missing `snapshot` elements in `StructureDefinition` resources
- Rewrites profile references within the package to be version-specific

These steps improve compatibility and are applied by default. Skipping them (`--skip-preprocessing`) preserves original resource IDs and structure, which can be useful if you need exact fidelity to the published package, but may cause reference resolution issues.

---

## Errors & troubleshooting

**How do I save error details to a file for later review?**

Use `--errors-output`:

```shell
# Save OperationOutcome files to the default system directory
$ kodjin-cli --errors-output=directory install hl7.fhir.us.core@4.0.0

# Save to a specific folder
$ kodjin-cli --errors-output=/tmp/kodjin-errors install hl7.fhir.us.core@4.0.0
```

---

**Requests to my server are timing out. How do I increase the timeout?**

Use `--request-timeout` to set a custom timeout in seconds (default is 30):

```shell
$ kodjin-cli --request-timeout 120 install hl7.fhir.us.core@4.0.0
```

---

**Installation is slow. How can I speed it up?**

Increase the number of parallel search requests with `--parallel-search-requests` (default is 10). Higher values can reduce wall-clock time but may increase server load:

```shell
$ kodjin-cli install --parallel-search-requests 20 hl7.fhir.us.core@4.0.0
```
