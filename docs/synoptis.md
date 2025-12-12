# Synopsis

Kodjin CLI follows a simple syntax pattern:

```shell
kodjin-cli [OPTIONS] <COMMAND> [OPTIONS]
```
Where

- `<COMMAND>`: Specifies the operation that you want to perform
- `[OPTIONS]`: Specifies configuration for the operation. Kodjin-cli has options for kodjin-cli and options for each operation of kodjin-cli.

Here’s a breakdown of the primary commands:
```shell
$ kodjin-cli --version
$ kodjin-cli server add https://demo.kodjin.com/fhir
$ kodjin-cli metadata
$ kodjin-cli info hl7.fhir.us.core@4.0.0
```

If you need help, run `kodjin-cli --help` from the terminal window.
