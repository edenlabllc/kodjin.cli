# server

This command allows you to add, list, view, and remove FHIR server URLs, making it easy to configure different environments for IG installation and validation.

Syntax:
```shell
kodjin-cli server <COMMAND>
```

## add      
The `add` command allows you to register a new FHIR server URL in the configuration, making it available for IG installation. This is useful when working with multiple FHIR environments, such as local development, staging, or production servers.

- Server URLs must be valid and accessible to avoid connection issues.
- If --name is not provided, the server will be listed using its URL.
- Use `kodjin-cli server list` to see all configured servers.
- If a server with the same URL already exists, the command will return an error.

Syntax:
```bash
kodjin-cli server add [OPTIONS] <URL>
```

Examples:

```shell
$ kodjin-cli server add https://demo.kodjin.com/fhir
Added server https://demo.kodjin.com/fhir running Kodjin FHIR Server v4.5.0
```

Options:

- -n, --name <NAME\> - set a name for the server 

example:
```shell
$ kodjin-cli server add --name DEV https://develop.com/fhir
Added server DEV running Kodjin FHIR Server develop
```

## default  
The `default` command sets one of the previously added FHIR servers as the default. Once a server is set as default, you do not need to specify the server URL every time you run commands related to IG installation and validation, simplifying workflows.

- The specified server must already be added using the server `add command`.
- Setting a default server eliminates the need to specify --server <server-url> in commands.
- You can change the default server at any time by running the command again with a different URL.
- Use `kodjin-cli server list` to check which server is currently set as default.

Syntax:
```shell
kodjin-cli server default <NAME>
```

Examples:
```shell
$ kodjin-cli server default https://demo.kodjin.com/fhir
Setting https://demo.kodjin.com/fhir as the default server
```

Options:

- -n, --name <NAME\> - set a name for the server 
  
## help
The `help` command returns help of the given subcommand(s)

Syntax:
```shell
kodjin-cli server --help
```

Examples:
```shell
$ kodjin-cli server --help
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

- The default server (if set) is clearly marked in the output.
- The name of the server is written with server domain in the brackets

Syntax:
```shell
kodjin-cli server list
```

Examples:
```shell
$ kodjin-cli server list
List of currently configured servers:
- https://example.fhir.server/r4
- https://demo.kodjin.com/fhir (default)
- DEV (https://develop.com/fhir)
```

## remove
The `remove` command removes a previously added FHIR server from the configured list. This is useful for cleaning up unused or outdated server entries.

- The server must already exist in the configured list, otherwise kodjin-cli will return an error.
- If the server being removed is the default server, you may need to set a new default using `kodjin-cli server default`.
- Use `kodjin-cli server list` to check which server is currently set as default.
  
Syntax:
```shell
kodjin-cli server remove <NAME>
```

Examples:
```shell
$ kodjin-cli server remove https://demo.kodjin.com/fhir
Removed server https://demo.kodjin.com/fhir
```

Options:

- -n, --name <NAME\> - set a name for the server 
