# Kodjin cli documentation

## Contents

- [Overview](#overview)
- [Installation](#installation)
- [Update](#update)
- [Getting Started](#getting-started)
- [Synopsis](#synopsis)
- [Basic options](#basic-options)
   - [-s, --server \<SERVER\>](#-s---server-server)
   - [--insecure-certificates](#--insecure-certificates)
   - [--request-timeout <REQUEST_TIMEOUT>](#--request-timeout-request_timeout)
   - [--errors-output <ERRORS_OUTPUT>](#--errors-output-errors_output)
   - [-h, --help](#-h---help)
   - [-V, --version](#-v---version)
- [Comands](#comands)
    - [server](#server)
        - [Options for server](#options-for-server)
    - [metadata](#metadata)
    - [install](#install)
    - [uninstall](#uninstall)
    - [check](#check)
    - [tree](#tree)
    - [info](#info)
    - [download](#download)
    - [generate-completions](#generate-completions)
    - [help](#help)

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
- `[OPTIONS]`: Specifies configuration for the operation. Kodjin-cli has basic options and options for each operation.

If you need help, run `kodjin-cli help` from the terminal window.


## Basic options

The Kodjin-cli supports several commands for managing IGs. Below is a list of commands with descriptions and examples.

### -s, --server \<SERVER\>

Select which FHIR server to use. The default one will be used if not specified

### --insecure-certificates

Skip TLS certificate validation

### --request-timeout <REQUEST_TIMEOUT>

Timeout for requests (in seconds) [default: 30]

### --errors-output <ERRORS_OUTPUT>

Where errors should be written to [default: stderr] [possible values: stderr, directory]

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

## Comands

### server

Manage FHIR server URLs

#### Options for server

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


