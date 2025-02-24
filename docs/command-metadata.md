# Command metadata

The `metadata` command retrieves and displays some data from the capability statement (server metadata) of a configured FHIR server. This provides essential details about the server's supported FHIR version, available resources, operations, and extensions.


**Usage:**
```shell
kodjin-cli metadata
```

**Examples:**

```shell
➜  $ kodjin-cli metadata

Name: Kodjin FHIR server 4.0.1 CapabilityStatement
Publisher: EdenLab
URL: https://demo.kodjin.com/fhir/metadata
Date: 2025-02-11T15:14:55.092247396+00:00
Software: Kodjin FHIR Server
Software Version: v4.7.0
FHIR Version: 4.0.1
```
