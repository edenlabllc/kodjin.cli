# Command check

The `check` command verifies whether a specific FHIR package is installed on a FHIR server. This helps users ensure that required Implementation Guides (IGs) and dependencies are available before proceeding with further operations.

Usage:
```shell
kodjin-cli check [OPTIONS] <NAME>
```

**Example:**

Check if a package is installed on the default server

```shell
  ~ kodjin-cli check hl7.fhir.us.core@4.0.0
```
In response kodjin-cli will return conformance resources that are missing
<details>
<summary> Response from the kodjin-cli to the check command</summary>

```shell
    ⠁ hl7.fhir.us.core@4.0.0: Fetching package info
    The following files are missing:
    ValueSet:
    - ValueSet-us-core-vital-signs.json (http://hl7.org/fhir/us/core/ValueSet/us-core-vital-signs|4.0.0)
    SearchParameter:
    - SearchParameter-us-core-device-patient.json (http://hl7.org/fhir/us/core/SearchParameter/us-core-device-patient|4.0.0)
    us-core-allergyintolerance-clinical-status|4.0.0)
    - SearchParameter-us-core-patient-gender.json (http://hl7.org/fhir/us/core/SearchParameter/us-core-patient-gender|4.0.0)
    CodeSystem:
    - CodeSystem-condition-category.json (http://hl7.org/fhir/us/core/CodeSystem/condition-category|4.0.0)
    Package hl7.fhir.us.core@4.0.0 is partially installed (39/148 resources present)
```
</details>

Notes

- This command helps prevent redundant installations and identify missing dependencies.
- Missing package and/or resources could be installed with  `kodjin-cli install`. 

Options:

## --type

**Usage:**
```shell
kodjin-cli check --type directory <PATH>
```

Available Options:

- `package` - checks resources in FHIR package from the official registry. This option is enabled by default, so it does not need to be explicitly specified when running the command.
- `directory` - checks resources in FHIR package from the specified directory. The `.` could be used to install package from the current directory.

**Examples:**

Here we will check if the FHIR package from /path/to/package directory exists
```shell
kodjin-cli check --type directory /path/to/package
```

In this example we will check if the package from the current directory exists
```shell
kodjin-cli check --type directory .
```

## --registry

Allows you to specify a custom FHIR package registry. By default, https://packages.simplifier.net is used for retrieving FHIR packages.
  
**Usage:**
```shell
kodjin-cli check --registry <REGISTRY>
```

Notes

- The specified registry must follow the [FHIR package specification](http://hl7.org/fhir/packages.html).
- If no `--registry` option is provided, the default registry is used.
- This option is useful for organizations that maintain private or custom FHIR package repositories.

## --help

The `help` command returns help of the given subcommand(s)

**Usage:**
```shell
kodjin-cli check --help
```

**Examples:**

```shell
$ kodjin-cli check --help
Check if a FHIR package is installed

Usage: kodjin-cli check [OPTIONS] <NAME>

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

  -h, --help
          Print help (see a summary with '-h')
```
