# Command reindex

The `reindex` command triggers reindexing of all resources on the selected FHIR server and waits until the operation finishes.

**Usage:**
```shell
kodjin-cli reindex
```

**Example:**
```shell
$ kodjin-cli reindex
```

Notes

- The command starts a reindex job on the server, then checks its status until it is completed.
- If the server reports an error status for the reindex job, the command fails.
- You can choose a specific configured server with the global `--server` option.

Options:

## --help

Shows help information for the `reindex` command.

**Usage:**
```shell
kodjin-cli reindex --help
```

**Example:**
```shell
$ kodjin-cli reindex --help
Trigger a reindex of all resources on the FHIR server

Usage: kodjin-cli reindex

Options:
  -h, --help
          Print help (see a summary with '-h')
```
