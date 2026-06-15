# Command install

The `install` command installs a FHIR package, either from the official FHIR package registry or from a local directory. This allows users to easily retrieve and use FHIR Implementation Guides (IGs) and dependencies.

!!! note "It is always better to use package to install conformance resources"

**Usage:**
```shell
kodjin-cli install [OPTIONS] <NAME>
```
**Examples:**

Install us-core package version 4.0.0
```shell
kodjin-cli install hl7.fhir.us.core@4.0.0
```

Options:

## --reindex

Triggers the `reindex` command automatically after a successful install.

**Examples:**
```shell
kodjin-cli install --reindex hl7.fhir.us.core@4.0.0
```

## --type

The --type option specifies the source type of the FHIR package when using the install command. It allows users to install packages either from the official FHIR package registry or from a local directory.

**Usage**:
```shell
kodjin-cli install --type <package|directory> <PATH>
```

Available Options:

- `package` (default) - retrieves FHIR package from the official registry. 
    - For example, you can install a US Core package version 4.0.0 `hl7.fhir.us.core@4.0.0`  
    - This is the default behavior, so it does not need to be explicitly specified.  
- `directory` - retrieves FHIR package from the specified directory. 
    - The `.` could be used to install package from the current directory.
- `file` - installs only specific files from a package or directory instead of the entire package.
    - Multiple files can be specified separated by spaces.
    - Useful when you only need certain conformance resources rather than the full package.

**Examples:**

Install package from the FHIR registry
```shell
kodjin-cli install hl7.fhir.us.core
```

Here we will install the FHIR package from /path/to/package directory
```shell
kodjin-cli install --type directory /path/to/package
```

In this example we will install the package from the current directory.
```shell
kodjin-cli install --type directory .
```

Install specific files from a package
```shell
kodjin-cli install --type file StructureDefinition-Patient.json ValueSet-Gender.json
```

Notes

- If no `--type` option is provided, `package` is used by default.
- Installing from a directory is useful when working with modified or custom packages.
- Ensure that the directory contains a valid FHIR package structure.
- The file option allows selective installation of specific FHIR resources without downloading the entire package. When using --type file, multiple filenames can be provided separated by spaces.

## --registry

Allows you to specify a custom FHIR package registry. By default, https://packages.simplifier.net is used for retrieving FHIR packages.

**Usage:**
```shell
kodjin-cli install --registry <REGISTRY>
```

**Examples:**

Install package from the default package regisrty
```shell
kodjin-cli install hl7.fhir.us.core@4.0.0
```

Install package from custom package regisrty
```shell
kodjin-cli install hl7.fhir.us.core@4.0.0 --registry https://custom.fhir.registry.com
```

Notes

- The specified registry must follow the [FHIR package specification](http://hl7.org/fhir/packages.html).
- If no `--registry` option is provided, the default registry is used.
- This option is useful for organizations that maintain private or custom FHIR package repositories.

## --existing-resources

Defines how kodjin-cli should handle resources that already exist on the FHIR server when installing a package. This option is particularly useful for subsequent package installations.

**Usage:**
```shell
kodjin-cli install <NAME> --existing-resources <skip|sync|overwrite>
```

Possible values:

- `skip` - When using skip, kodjin-cli will skip resources that are already installed on the server. Is a dedault value.
- `sync` - Update existing resources if they are different from what's being installed
- `overwrite` - When using overwrite kodjin-cli will update resources, if they are already exist on the server

**Examples:**

Install a package and overwrite existing resources
```shell
$ kodjin-cli install hl7.fhir.us.core@4.0.0 --existing-resources overwrite
```

Install a package and sync existing resources
```shell
$ kodjin-cli install hl7.fhir.us.core@4.0.0 --existing-resources sync
```

Notes

- `skip` is default value, so it does not need to be explicitly specified when running the command.
- The `overwrite` option should be used cautiously, especially in production environments, as it will replace existing resources.
- The `sync` option provides a middle ground by only updating resources when they differ from the package version.
- To check what resources are already installed, use `kodgin-cli install us.core --preprocess`.


## --skip-strict-reference-versions

Disables the enforcement of version-specific references when installing FHIR packages. This option should be used with caution, as references in FHIR resources often include versions to ensure compatibility and consistency.

**Usage:**
```shell
kodjin-cli install --skip-strict-reference-versions <NAME>
```

When installing packages, kodjin-cli performs several updates to conformance resources, including:

- Generating new resource IDs for canonical resources (those with a url and version).
- Generating missing snapshots for StructureDefinition resources.
- Making references in StructureDefinition resources version-specific within the package.

By using --skip-strict-reference-versions, Kodjin CLI does not enforce version-specific references, potentially allowing references to resolve more flexibly.

**Examples:**
```shell
$ kodjin-cli install --skip-strict-reference-versions ihe.formatcode.fhir@1.1.0
```

Notes

- Use this option carefully, as improper reference handling may cause validation or compatibility issues.
- For more details on FHIR references and canonical URLs, refer to the FHIR documentation.

## --skip-dependencies

Skips installing dependencies. By default, when installing packages, kodjin-cli also installs all missing dependencies. To view these dependencies, users can use the `kodjin-cli tree` command.

**Examples:**
```shell
$ kodjin-cli install --skip-dependencies hl7.fhir.us.core@4.0.0
hl7.fhir.us.core@4.0.0: installing 109 resources
```

Notes

-  Use this option carefully, as missing dependencies may cause validation errors.

## --skip-preprocessing

Disables resource preprocessing during package installation. This option preserves the original state of resources as they exist in the package, bypassing the automatic modifications that kodjin-cli normally applies.

**Usage:**
```shell
kodjin-cli install --skip-preprocessing <NAME>
```

By default, kodjin-cli performs preprocessing on resources during installation, which includes:

- Generating new resource IDs for canonical resources (those with a url and version)
- Generating missing snapshots for StructureDefinition resources
- Making references to other profiles within the current package version-specific in StructureDefinition resources

When `--skip-preprocessing` is used, these automatic modifications are bypassed, and resources are installed exactly as they appear in the original package.

**Examples:**
```shell
$ kodjin-cli install --skip-preprocessing hl7.fhir.us.core@4.0.0
```

Notes

- This option is useful when you want to preserve original resource IDs or maintain the exact structure of the package as published.
- Use this option carefully, as skipping preprocessing may result in missing snapshots or non-version-specific references, which could affect validation or compatibility.
- Consider the implications of preserving original resource IDs, especially in environments where ID conflicts might occur.

## --parallel-search-requests <PARALLEL_SEARCH_REQUESTS>

Specifies how many search requests can be performed in parallel when checking package files. This can improve performance by speeding up operations that involve remote lookups or validations.

**Examples**
```shell
$ kodjin-cli install --parallel-search-requests 8 hl7.fhir.us.core@4.0.0
```

Notes

- A higher number may speed up processing but can increase system load or trigger rate limits, depending on your environment.
- Default number is 10

## --help

The `help` command returns help of the given subcommand(s)

**Usage:**
```shell
kodjin-cli install --help
```

**Examples:**

```shell
$ kodjin-cli install --help
Install a FHIR package

Usage: kodjin-cli install [OPTIONS] <NAME>...

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

  -r, --reindex
          Trigger a reindex after package installation completes

  -g, --registry <REGISTRY>
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
