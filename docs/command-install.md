# Command install

The `install` command installs a FHIR package, either from the official FHIR package registry or from a local directory. This allows users to easily retrieve and use FHIR Implementation Guides (IGs) and dependencies.

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

Notes

- If no `--type` option is provided, `package` is used by default.
- Installing from a directory is useful when working with modified or custom packages.
- Ensure that the directory contains a valid FHIR package structure.

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
kodjin-cli install <NAME> --existing-resources <skip|overwrite>
```

Possible values:
- `skip` - When using skip, kodjin-cli will skip resources that are already installed on the server. Is a dedault value.
- `overwrite` - When using overwrite kodjin-cli will update resources, if they are already exist on the server

**Examples:**

Install a package and overwrite existing resources
```shell
$ kodjin-cli install hl7.fhir.us.core@4.0.0 --existing-resources overwrite
```

Notes

- `skip` is default value, so it does not need to be explicitly specified when running the command.
- The `overwrite` option should be used cautiously, especially in production environments, as it will replace existing resources.
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

By using --skip-strict-reference-versions, the Kodjin CLI does not enforce version-specific references, potentially allowing references to resolve more flexibly.

**Examples:**
```shell
$ kodjin-cli install --skip-strict-reference-versions ihe.formatcode.fhir@1.1.0
```

Notes

- Use this option carefully, as improper reference handling may cause validation or compatibility issues.
- For more details on FHIR references and canonical URLs, refer to the FHIR documentation.


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

  -h, --help
          Print help (see a summary with '-h')
```
