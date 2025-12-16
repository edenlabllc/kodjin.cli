# Command help

Displays help information for kodjin-cli, including available commands, options, and usage examples. This command return the same information as [option `--help`](../global-options/#-h-help)


**Usage:**
```shell
kodjin-cli help
```

<b>Examples:</b>

```shell
$ kodjin-cli help
Kodjin management CLI

Usage examples:
$ kodjin-cli server add https://demo.kodjin.com/fhir
$ kodjin-cli metadata
$ kodjin-cli info de.gematik.epa
$ kodjin-cli install hl7.fhir.us.core@4.0.0
$ kodjin-cli --errors-output=directory install hl7.fhir.us.core@4.0.0
$ kodjin-cli --server=kodjin-demo check hl7.fhir.us.core@4.0.0

For full information, see --help for each subcommand.

Usage: kodjin-cli [OPTIONS] <COMMAND>

Commands:
  server                Manage FHIR server URLs
  metadata              Show FHIR server metadata
  install               Install a FHIR package
  uninstall             Uninstall a FHIR package
  check                 Check if a FHIR package is installed
  tree                  Print dependency tree of a FHIR package
  info                  Show information about a FHIR package
  download              Download a package locally
  generate-completions  Generate command autocompletions
  help                  Print this message or the help of the given subcommand(s)

Options:
  -s, --server <SERVER>
          Select which FHIR server to use. The default one will be used if not specified

      --insecure-certificates
          Skip TLS certificate validation

      --request-timeout <REQUEST_TIMEOUT>
          Timeout for requests (in seconds)

          [default: 30]

      --errors-output <ERRORS_OUTPUT>
          Where errors should be written to.

          Can be either `stderr` (default), `directory` for the default logs directory or a custom path.

          [default: stderr]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
