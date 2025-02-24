# Command download

The `download` command retrieves a FHIR package from the registry and saves it locally. This allows you to inspect and modify conformance resources before installing them on a FHIR server.

Note! "`download` command always save files in the current directory"

**Usage**
```shell
 kodjin-cli download [OPTIONS] <NAME>
```

**Examples**

Download all files from the package 
```shell
$ kodjin-cli download hl7.fhir.uv.extensions.r4@5.1.0
Package downloaded to hl7.fhir.uv.extensions.r4@5.1.0
```

Optaions:

## --preprocess

The option `--preprocess` allow you to perform all actions that is done by the install command. This could be usefull if you want to wark with conformance resources locally.

When installing packages, kodjin-cli performs several updates to conformance resources, including:

- Generating new resource IDs for canonical resources (those with a url and version).
- Generating missing snapshots for StructureDefinition resources.
- Making references in StructureDefinition resources version-specific within the package.

**Usage**
```shell
 kodjin-cli download --preprocess <NAME>
```

**Examples**

Download all files from the package 
```shell
$ kodjin-cli download --preprocess hl7.fhir.uv.extensions.r4@5.1.0
Preprocessed file package/SearchParameter-us-core-immunization-patient.json
Note: 35 profile reference fields were normalized to contain an explicit version in profile package/StructureDefinition-pediatric-bmi-for-age.json
Preprocessed file package/StructureDefinition-pediatric-bmi-for-age.json
...
Package downloaded to hl7.fhir.us.core@4.0.0
```


