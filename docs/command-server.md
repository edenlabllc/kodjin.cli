# Command server

This command allows you to add, list, view, and remove FHIR server URLs, making it easy to configure different environments for IG installation and validation.

**Usage:**
```shell
kodjin-cli server <COMMAND> [OPTIONS]
```

Subcommand that could be used with command `server`:

## add      
The `add` command allows you to register a new FHIR server URL in the configuration, making it available for IG installation. This is useful when working with multiple FHIR environments, such as local development, staging, or production servers.

**Usage:**
```shell
kodjin-cli server add [OPTIONS] <URL>
```

**Examples:**

```shell
$ kodjin-cli server add https://develop.com/fhir
Added server https://develop.com/fhir running Kodjin FHIR Server v4.5.0
```

Options:

- -n, --name <NAME\> - set a name for the server 

**Examples:**
```shell
$ kodjin-cli server add --name DEV https://develop.com/fhir
Added server DEV running Kodjin FHIR Server develop
```

Notes

- Server URLs must be valid and accessible to avoid connection issues.
- If --name is not provided, the server will be listed using its URL.
- Use `kodjin-cli server list` to see all configured servers.
- If a server with the same URL already exists, the command will return an error.

## default  
The `default` command sets one of the previously added FHIR servers as the default. Once a server is set as default, you do not need to specify the server URL every time you run commands related to IG installation and validation, simplifying workflows.

**Usage:**
```shell
kodjin-cli server default <NAME>
```

Options:

- -n, --name <NAME\> - set a name for the server 
  

**Examples:**

Set default server
```shell
$ kodjin-cli server default https://develop.com/fhir
Setting https://develop.com/fhir as the default server
```

Set default server, using the Name
```shell
$ kodjin-cli server default DEVELOP
Setting DEVELOP as the default server
```

Notes

- The specified server must already be added using the server `add command`.
- Setting a default server eliminates the need to specify --server <server-url> in commands.
- You can change the default server at any time by running the command again with a different URL.
- Use `kodjin-cli server list` to check which server is currently set as default.

## help
The `help` command returns help of the given subcommand(s)

**Usage:**
```shell
kodjin-cli server help
```

**Examples:**
```shell
$ kodjin-cli server help
Manage FHIR server URLs

Usage: kodjin-cli server <COMMAND>

Commands:
  list     List currently configured servers
  add      Add a new FHIR server with the provided URL
  remove   Remove a FHIR server
  default  Set a FHIR server as the default
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## list
The `list` command displays all currently configured FHIR servers, including the default server (if set). This helps users quickly view available servers and manage their configurations.

**Usage:**
```shell
kodjin-cli server list
```

**Examples:**
```shell
$ kodjin-cli server list
List of currently configured servers:
- https://example.fhir.server/r4
- https://demo.kodjin.com/fhir (default)
- DEV (https://develop.com/fhir)
```

Notes

- The default server (if set) is clearly marked in the output.
- The name of the server is written with server domain in the brackets

## remove
The `remove` command removes a previously added FHIR server from the configured list. This is useful for cleaning up unused or outdated server entries.

**Usage:**
```shell
kodjin-cli server remove <NAME>
```

Options:

- -n, --name <NAME\> - set a name for the server 

**Examples:**
```shell
$ kodjin-cli server remove https://demo.kodjin.com/fhir
Removed server https://demo.kodjin.com/fhir
```

Notes

- The server must already exist in the configured list, otherwise kodjin-cli will return an error.
- If the server being removed is the default server, you may need to set a new default using `kodjin-cli server default`.
- Use `kodjin-cli server list` to check which server is currently set as default.
  