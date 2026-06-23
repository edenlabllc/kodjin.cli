---
description: How to capture and review error details from Kodjin CLI operations.
---

# Save error logs to a file

When Kodjin CLI installs or checks a package, the FHIR server may return error responses for individual resources — for example, if a resource failed validation or conflicted with something already on the server. These responses are called **OperationOutcomes**.

By default, OperationOutcomes are printed directly to the terminal as they occur. For large installs with many resources, this can be hard to review. You can redirect them to files instead.

---

## Default: errors print to the terminal

With no extra flags, all output — including errors — goes to the terminal:

```shell
$ kodjin-cli install hl7.fhir.us.core@4.0.0
```

This is fine for small installs or when you're watching the output live. For bigger packages, the errors can scroll past quickly and be difficult to read.

---

## Save errors to the default directory

Add `--errors-output=directory` to save OperationOutcome responses as `.ndjson` files in the standard application data directory for your platform:

```shell
$ kodjin-cli --errors-output=directory install hl7.fhir.us.core@4.0.0
```

The terminal still shows progress (which files are being processed), but detailed error payloads are written to disk. The default save location depends on your operating system:

| Platform | Location |
|---|---|
| macOS | `~/Library/Application Support` |
| Linux | `~/.local/share` |
| Windows | `%LOCALAPPDATA%` |

---

## Save errors to a specific folder

If you want to control exactly where the files go, provide a folder path instead:

```shell
# Save to a folder called "kodjin-errors" in /tmp
$ kodjin-cli --errors-output=/tmp/kodjin-errors install hl7.fhir.us.core@4.0.0

# Save to the current directory
$ kodjin-cli --errors-output=. install hl7.fhir.us.core@4.0.0
```

The folder will be created if it doesn't exist. Each resource that produced an error gets its own `.ndjson` file, making it easy to look up a specific failure.

---

## When to use this

- **Large installs** — when hundreds of resources are being uploaded and you want a record to review later
- **Debugging failures** — when an install reports errors and you need the full OperationOutcome details to understand what went wrong
- **Auditing** — when you need to keep a log of what the server rejected for compliance or review purposes
