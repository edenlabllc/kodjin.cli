# Kodjin cli documentation

## Contents

- [Kodjin cli documentation](#kodjin-cli-documentation)
    - [Contents](#contents)
    - [Overview](#overview)
    - [Installation](#installation)
    - [Update](#update)
    - [Getting Started](#getting-started)
    - [Synopsis](#synopsis)
    - [Command auto-completion](#command-auto-completion)
    - [Global options](#global-options)
        - [-s, --server \<SERVER\>](#-s---server-server)
        - [--insecure-certificates](#--insecure-certificates)
        - [--request-timeout \<REQUEST\_TIMEOUT\>](#--request-timeout-request_timeout)
        - [--errors-output \<ERRORS\_OUTPUT\>](#--errors-output-errors_output)
        - [-h, --help](#-h---help)
        - [-V, --version](#-v---version)
    - [Comands](#comands)
        - [server](#server)
            - [add](#add)
            - [default](#default)
            - [help](#help)
            - [list](#list)
            - [remove](#remove)
        - [metadata](#metadata)
        - [install](#install)
        - [uninstall](#uninstall)
        - [check](#check)
        - [tree](#tree)
        - [info](#info)
        - [download](#download)
        - [generate-completions](#generate-completions)
        - [help](#help-1)

## Overview

The `Kodjin cli` is a command-line tool designed to simplify the process of downloading, installing, and managing FHIR Implementation Guides (IGs) for FHIR servers. The tool automates fetching IGі and its dependencies, ensuring compatibility and efficient setup.

## Installation

TBD

## Update

TBD

## Getting Started

Once installed, you can verify the installation by running:

```bash
kodjin-cli --version
```

## Synopsis

The Kodjin-cli follows a simple syntax pattern:

```bash
kodjin-cli [OPTIONS] <COMMAND> [OPTIONS]
```
Where
- `<COMMAND>`: Specifies the operation that you want to perform
- `[OPTIONS]`: Specifies configuration for the operation. Kodjin-cli has options for kodjin-cli and options for each operation of kodjin-cli.

Here’s a breakdown of the primary commands:
```bash
$ kodjin-cli --version
$ kodjin-cli server add https://demo.kodjin.com/fhir
$ kodjin-cli metadata
$ kodjin-cli info hl7.fhir.us.core@4.0.0
```

If you need help, run `kodjin-cli --help` from the terminal window.

## Command auto-completion

The Kodjin-cli includes a command auto-completion feature that enables you to use the **Tab** key to complete a partially entered command.

How it works?

When you enter a part of the command after a `kodjin-cli` you can press **Tab** and command auto-completion either automatically completes your command or displays a suggested list of commands.

For example, if you enter `kodjin-cli in` and press **Tab** then kodjin-cli will suggest you options to choose
```bash
$ kodjin-cli in[Tab]
info     -- Show information about a FHIR package
install  -- Install a FHIR package
```  

If you enter `kodjin-cli ins` and press **Tab** then kodjin-cli will autocomplete the command
```bash 
$ kodjin-cli install
```

## Global options

When running kodjin-cli, you can specify global options to customize its behavior. For instance, you may choose an output folder for logs or define a specific server if it differs from the default. Below is a list of available options, along with descriptions and usage examples.

In case opation uses an argument they should be separated with a space. In this documentation arguments are mentioned in angle brackets 

### -s, --server \<SERVER\>

If you want to use server that differ from defaulf you can add `--server` option. To see the default version of the server use [server list command](#list)

Syntax:
```bash
$ kodjin-cli --server <server>
```

Example:

In this example we will check what is the default sever is and then use for installing IG to the one that is not default
```bash
$ kodjin-cli server list

List of currently configured servers:
- https://production.com/fhir (default)
- https://develop.com/fhir

$ kodjin-cli --server https://develop.com/fhir install hl7.fhir.us.core@4.0.0
```

If you want to instal IG to your default server https://production.com/fhir`, then you do not need to use this flag


### --insecure-certificates

Skips TLS (Transport Layer Security) certificate validation, allowing connections to servers with self-signed or invalid certificates. 

> Note: This option should be used with caution and only in trusted development or testing environments, as it bypasses a key security measure.

Syntax:
```bash
$ kodjin-cli --insecure-certificates
```

Example:

In this example, we will install the specified Implementation Guide without verifying the server's TLS certificate.
```bash
$ kodjin-cli --insecure-certificates install hl7.fhir.us.core@4.0.0
```

### --request-timeout <REQUEST_TIMEOUT>

Limit the waiting time of the response to REQUEST_TIMEOUT (in seconds). The default value is 30 sec. If you want to wait more or less you can change this value by adding `--request-timeout` to the request.

Syntax:
```bash
$ kodjin-cli --request-timeout <number>
```

Example:

In this example, we will increase waiting time for the response
```bash
$ kodjin-cli --request-timeout 40 install hl7.fhir.us.core@4.0.0
```


### --errors-output <ERRORS_OUTPUT>

Specifies the output location for error logs generated when working with Implementation Guides (IGs). This option allows you to control where error messages are displayed or saved.

Available Options:

- `stderr` – processed files, Implementation Guides (IGs), OperationOutcomes are written directly to the console. This is the default value.
- `directory` – titles of IGs and files processed are written to the console, but OperationOutcomes are saved in newline-delimited JSON (.ndjson) files within the default directory.

Each system has its own default directory:
| Platform | Value                                | Example                                         |
| -------- | ------------------------------------ | ----------------------------------------------- |
| Linux    | $XDS_DATA_HOME or $HOME/.local/share | /home/\<username\>/.local/share                 |
| macOS    | $HOME/Library/Application Support    | /Users/\<username\>/Library/Application Support |
| Windows  | {FOLDER_LocalAppData}                | C:\Users\\<username\>\AppData\Local             |

-  `folder path` - instead of writing OperationOutcome .ndjson files to the default directory you can choose any directory that is 

Syntax:
```bash
$ kodjin-cli --errors-output=<stderr|directory|folder path> <command>
```

Examples:

As `stderr` is a default value to use this value we do not need to add an option

```bash
$ kodjin-cli install hl7.fhir.au.base@4.2.2-ballot
```

### -h, --help

Print help (see a summary with '-h')

Example for summary:

```bash
➜ kodjin-cli -h
```

Response example for summary

```bash
Kodjin management CLI

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
          Timeout for requests (in seconds) [default: 30]
      --errors-output <ERRORS_OUTPUT>
          Where errors should be written to [default: stderr] [possible values: stderr, directory]
  -h, --help
          Print help (see more with '--help')
  -V, --version
          Print version
```

</details>

Example for full help information: 

```bash
kodjin-cli --help
```
Respone for for full help information: 
```bash
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
          Where errors should be written to

          [default: stderr]
          [possible values: stderr, directory]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

### -V, --version

Print version

Examples:

```bash
$ kodjin-cli --version
kodjin-cli 0.1.0
```

## Comands

The Kodjin-cli supports several commands for managing IGs. Below is a list of commands with descriptions and examples.

### server

Manage FHIR server URLs

#### add      
-- Add a new FHIR server with the provided URL

Examples:

```bash
$ kodjin-cli server add https://demo.kodjin.com/fhir
Added server https://demo.kodjin.com/fhir running Kodjin FHIR Server v4.5.0
```

#### default  
-- Set a FHIR server as the default

Examples:
```bash
$ kodjin-cli server default https://demo.kodjin.com/fhir
Setting https://demo.kodjin.com/fhir as the default server
```

#### help
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

#### list
-- List currently configured servers

Examples:
```bash
$ kodjin-cli server list
List of currently configured servers:
- https://example.fhir.server/r4
- https://demo.kodjin.com/fhir (default)
```

#### remove
-- Remove a FHIR server

Examples:
```bash
$ kodjin-cli server remove https://demo.kodjin.com/fhir
Removed server https://demo.kodjin.com/fhir
```

### metadata

Show FHIR server metadata

### install

Install a FHIR package

### uninstall

Uninstall a FHIR package

### check

Check if a FHIR package is installed

### tree

Print dependency tree of a FHIR package

### info

Show information about a FHIR package

### download

Download a package locally

### generate-completions

Generate command autocompletions

### help

Print this message or the help of the given subcommand(s)


