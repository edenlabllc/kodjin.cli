### install

The `install` command installs a FHIR package, either from the official FHIR package registry or from a local directory. This allows users to easily retrieve and use FHIR Implementation Guides (IGs) and dependencies.

Syntax:
```shell
kodjin-cli install [OPTIONS] <NAME>
```

## --type

Syntax:
```shell
kodjin-cli install --type directory <PATH>
```

Available Options:

- `package` - retrieves FHIR package from the official registry. For example, you can install a US Core package version 4.0.0 `hl7.fhir.us.core@4.0.0` This option is enabled by default, so it does not need to be explicitly specified when running the command.
- `directory` - retrieves FHIR package from the specified directory. The `.` could be used to install package from the current directory.

Examples:
```shell
kodjin-cli install --type directory /path/to/package
```
> Installs the FHIR package from /path/to/package.

Examples:
```shell
kodjin-cli install --type directory .
```
> Installs the package from the current directory.

## --registry
The `registry` command 
Syntax:
```shell

```

## existing-resources

Syntax:
```shell

```

## skip-strict-reference-versions

Syntax:
```shell

```

## preprocess

Syntax:
```shell

```

## help

The `help` command returns help of the given subcommand(s)

Syntax:
```shell
kodjin-cli install [OPTIONS] <NAME>
```

Examples:

```shell
$ kodjin-cli install --help
Install a FHIR package

Usage: kodjin-cli install [OPTIONS] <NAME>

Arguments:
  <NAME>
          Item to process

Options:
  -t, --type <TYPE>
          Type of the item

          [default: package]

          Possible values:
          - package:   FHIR Package from a registry
          - directory: Local directory

  -r, --registry <REGISTRY>
          Registry URL for FHIR packages

          [default: https://packages.simplifier.net]

      --existing-resources <EXISTING_RESOURCES>
          What should be done with resources that already exist

          Note: this setting is not applied to dependencies in order to avoid accidentally overwriting resources.

          [default: skip]

          Possible values:
          - skip:      Skip existing resources
          - overwrite: Overwrite existing resources

      --skip-strict-reference-versions
          Do not change profile references to be version-specific, keep them as-is instead

      --preprocess
          Perform resource preprocessing that is normally done before installation

          Currently does the following: - Generates new resource ids for canonical resources (ones that have a url and version present) - Generates snapshots for StructureDefinition resources where they are missing - Makes references to other profiles within the current package in StructureDefinition resources version-specific

  -h, --help
          Print help (see a summary with '-h')
```
