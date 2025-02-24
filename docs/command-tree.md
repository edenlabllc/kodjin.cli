# Command tree

The `tree` command displays the dependency tree of a FHIR package, showing its relationships with other packages and dependencies. This helps users understand which other packages are required for a specific Implementation Guide (IG).

**Usage**

```shell
 kodjin-cli tree [OPTIONS] <NAME>
```

**Examples:**

Display the dependency tree for a package

```shell
$ kodjin-cli tree hl7.fhir.us.core@4.0.0

 - hl7.fhir.us.core@4.0.0 (148 resources)
   - hl7.fhir.uv.bulkdata@1.0.1 (6 resources)
     - hl7.fhir.r4.core@4.0.1 (4581 resources)
   - hl7.fhir.r4.core@4.0.1 (4581 resources)
   - us.nlm.vsac@0.3.0 (8916 resources)
```

Notes

- Dependencies are retrieved from the FHIR package registry.
- Understanding dependencies is useful when resolving compatibility issues or troubleshooting installations.
- Use `kodjin-cli install` command to install package with all dependencies.
- Package `hl7.fhir.r4.core@4.0.1` is a basic FHIR package and should already be installed on your FHIR server.
