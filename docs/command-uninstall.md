# Command uninstall

The `uninstall` command removes conformance resources associated with a FHIR package from a FHIR server. This allows users to clean up installed Implementation Guides (IGs) and dependencies when they are no longer needed.

**Usage:**
```shell
kodjin-cli uninstall [OPTIONS] <NAME>
```

**Examples:**

Uninstall a package from the default FHIR server
```shell
kodjin-cli uninstall hl7.fhir.us.core
```

Notes

- This command removes only conformance resources (e.g., StructureDefinition, CapabilityStatement, ValueSet, etc.).
- It does not delete patient data or other non-conformance resources.
- Use `kodjin-cli check` to check if package is already installed.


**Options**:

## --type

The --type option specifies the source type of the FHIR package when using the uninstall command. It allows users to uninstall either official FHIR packages or custom resources from a local directory.

**Usage**:
```shell
kodjin-cli uninstall --type <package|directory> <PATH>
```

Available Options:

- `package` (default) - retrieves FHIR package from the official registry. 
    - For example, you can uninstall a US Core package version 4.0.0 `hl7.fhir.us.core@4.0.0`  
    - This is the default behavior, so it does not need to be explicitly specified.  
- `directory` - retrieves FHIR package from the specified directory. 
    - The `.` could be used to uninstall package from the current directory.


**Examples:**

Uninstall package from the FHIR registry
```shell
kodjin-cli uninstall hl7.fhir.us.core
```

Here we will uninstall the FHIR package from /path/to/package directory
```shell
kodjin-cli uninstall --type directory /path/to/package
```

In this example we will uninstall the package from the current directory.
```shell
kodjin-cli install --type directory .
```

Notes

- If no `--type` option is provided, `package` is used by default.
- Uninstalling from a directory is useful when you worked with modified or custom packages and now want to revert those custom updates.
