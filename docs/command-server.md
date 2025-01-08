# server

Manage FHIR server URLs

## add      
-- Add a new FHIR server with the provided URL

Examples:

```bash
$ kodjin-cli server add https://demo.kodjin.com/fhir
Added server https://demo.kodjin.com/fhir running Kodjin FHIR Server v4.5.0
```

## default  
-- Set a FHIR server as the default

Examples:
```bash
$ kodjin-cli server default https://demo.kodjin.com/fhir
Setting https://demo.kodjin.com/fhir as the default server
```

## help
-- Print this message or the help of the given subcommand(s)

Examples:
```bash
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
-- List currently configured servers

Examples:
```bash
$ kodjin-cli server list
List of currently configured servers:
- https://example.fhir.server/r4
- https://demo.kodjin.com/fhir (default)
```

## remove
-- Remove a FHIR server

Examples:
```bash
$ kodjin-cli server remove https://demo.kodjin.com/fhir
Removed server https://demo.kodjin.com/fhir
```
