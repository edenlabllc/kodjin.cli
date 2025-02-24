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

