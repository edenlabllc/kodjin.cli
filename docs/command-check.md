# Command check

The `check` command verifies whether specific FHIR packages are installed on a FHIR server. This helps users ensure that required Implementation Guides (IGs) and dependencies are available before proceeding with further operations.

Usage:
```shell
kodjin-cli check [OPTIONS] <NAME>...
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
- `file` - checks a single local file.
  
**Examples:**

Here we will check if the FHIR package from /path/to/package directory exists
```shell
kodjin-cli check --type directory /path/to/package
```

In this example we will check if the package from the current directory exists
```shell
kodjin-cli check --type directory .
```

Check a single local file
```shell
kodjin-cli check --type file StructureDefinition-Patient.json
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

## --existing-resources

Specifies what should be done with resources that already exist during the check operation.

**Usage:**
```shell
kodjin-cli check --existing-resources <OPTION>
```

Available Options:

- `skip` - Skip existing resources (default)
- `sync` - Update existing resources if they are different from what's being checked
- `overwrite` - Always overwrite existing resources

**Note:** This setting is not applied to dependencies to avoid accidentally overwriting resources.

## --skip-strict-reference-versions

By default, the command makes profile references version-specific. This flag disables that behavior, keeping references as-is instead.

**Usage:**
```shell
kodjin-cli check --skip-strict-reference-versions
```

## --skip-dependencies

Prevents automatic checking of package dependencies. By default, dependencies are checked along with the main package.

**Usage:**
```shell
kodjin-cli check --skip-dependencies
```

## --parallel-search-requests <PARALLEL_SEARCH_REQUESTS>

Specifies how many search requests can be performed in parallel when checking package files. This can improve performance by speeding up operations that involve remote lookups or validations.

**Examples**
```shell
$ kodjin-cli check --parallel-search-requests 8 hl7.fhir.us.core@4.0.0
```

Notes

- A higher number may speed up processing but can increase system load or trigger rate limits, depending on your environment.
- Default number is 10

## --skip-preprocessing

Disables resource preprocessing during package installation. This option preserves the original state of resources as they exist in the package, bypassing the automatic modifications that kodjin-cli normally applies.

**Usage:**
```shell
kodjin-cli check --skip-preprocessing <NAME>
```

By default, kodjin-cli performs preprocessing on resources during installation, which includes:

- Generating new resource IDs for canonical resources (those with a url and version)
- Generating missing snapshots for StructureDefinition resources
- Making references to other profiles within the current package version-specific in StructureDefinition resources

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

Usage: kodjin-cli check [OPTIONS] <NAME>...

Arguments:
  <NAME>...
          Items to process

Options:
  -t, --type <TYPE>
          Type of the item

          [default: package]

          Possible values:
          - package:   FHIR Package from a registry
          - directory: Local directory
          - file:      Single local file

  -r, --registry <REGISTRY>
          Registry URL for FHIR packages

          [default: https://packages.simplifier.net]

      --existing-resources <EXISTING_RESOURCES>
          What should be done with resources that already exist

          Note: this setting is not applied to dependencies in order to avoid accidentally overwriting resources.

          [default: skip]

          Possible values:
          - skip:      Skip existing resources
          - sync:      Update existing resources if they are different from what's being installed
          - overwrite: Always overwrite existing resources

      --skip-strict-reference-versions
          Do not change profile references to be version-specific, keep them as-is instead

      --skip-dependencies
          Do not automatically install package dependencies

      --parallel-search-requests <PARALLEL_SEARCH_REQUESTS>
          How many search requests can be performed in parallel when checking package files

          [default: 10]

      --skip-preprocessing
          Skip resource preprocessing. This can be useful if you want to e.g. keep original resource IDs.

          Currently preprocessing does the following:

          - Generates new resource ids for canonical resources (ones that have a url and version present)

          - Generates snapshots for StructureDefinition resources where they are missing

          - Makes references to other profiles within the current package in StructureDefinition resources version-specific

  -h, --help
          Print help (see a summary with '-h')
```
