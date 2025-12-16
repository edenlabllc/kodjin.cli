# Global options

When running kodjin-cli, you can specify global options to customize its behavior. For instance, you may choose an output folder for logs or define a specific server if it differs from the default. Below is a list of available options, along with descriptions and usage examples.

In case option uses an argument they should be separated with a space. In this documentation arguments are mentioned in angle brackets.

## -s, --server <SERVER>

If you want to use server that differ from default you can add `--server` option. To see the default version of the server use [server list command](/command-server/#list)

**Usage:**
```shell
kodjin-cli --server <server>
```

**Example:**

In this example we will check what is the default server is and then use for installing IG to the one that is not default
```shell
$ kodjin-cli server list

List of currently configured servers:
- https://production.com/fhir (default)
- https://develop.com/fhir

$ kodjin-cli --server https://develop.com/fhir install hl7.fhir.us.core@4.0.0
```

If you want to install IG to your default server, then you do not need to use this flag.

## -H, --header <HEADER>

Adds a custom HTTP header to requests sent to the FHIR server. This option can be used multiple times to add several headers. Headers should be specified in the format `Header-Name: value`.

**Usage:**
```shell
kodjin-cli --header <header>
```

**Example:**

In this example, we will add a custom header to the request:
```shell
$ kodjin-cli --header "X-Custom-Header: custom-value" install hl7.fhir.us.core@4.0.0
```

To add multiple headers:
```shell
$ kodjin-cli --header "X-Custom-Header: value1" --header "X-Another-Header: value2" install hl7.fhir.us.core@4.0.0
```

## -a, --auth <AUTH>

Specifies the authentication type to use when connecting to the FHIR server. This option determines how kodjin-cli will authenticate with the server.

**Available authentication types:**

- `basic` – HTTP Basic Authentication
- `bearer` – Bearer Token Authentication
- `oauth` – OAuth2 (Client Credentials flow)

**Usage:**
```shell
kodjin-cli --auth <auth-type>
```

**Examples:**

Using Basic Authentication:
```shell
$ kodjin-cli --auth basic --user myusername --password mypassword install hl7.fhir.us.core@4.0.0
```

Using Bearer Token:
```shell
$ kodjin-cli --auth bearer --bearer mytoken123 install hl7.fhir.us.core@4.0.0
```

Using OAuth2:
```shell
$ kodjin-cli --auth oauth --token-url https://auth.example.com/token --client-id myclientid --client-secret myclientsecret install hl7.fhir.us.core@4.0.0
```
## HTTP Basic Authentication
### -u, --user <USER>

Specifies the username for authentication. This option is used in conjunction with `--auth basic` for HTTP Basic Authentication.

**Usage:**
```shell
kodjin-cli --auth basic --user <username>
```

### -p, --password <PASSWORD>

Specifies the password for authentication. This option is used in conjunction with `--auth basic` for HTTP Basic Authentication.

> Note: Be cautious when using passwords in command-line arguments, as they may be visible in command history or process listings.

**Usage:**
```shell
kodjin-cli --auth basic --password <password>
```

**Example:**

```shell
$ kodjin-cli --auth basic --user admin --password secretpass install hl7.fhir.us.core@4.0.0
```

## Bearer Token Authentication
### -b, --bearer <BEARER>

Specifies the bearer token for authentication. This option is used in conjunction with `--auth bearer` for Bearer Token Authentication.

**Usage:**
```shell
kodjin-cli --auth bearer --bearer <token>
```

**Example:**

```shell
$ kodjin-cli --auth bearer --bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9... install hl7.fhir.us.core@4.0.0
```

## OAuth2 (Client Credentials flow)
### --token-url <TOKEN_URL>

Specifies the OAuth2 token endpoint URL. This option is required when using `--auth oauth` for OAuth2 authentication with the Client Credentials flow.

**Usage:**
```shell
kodjin-cli --auth oauth --token-url <url>
```

### --client-id <CLIENT_ID>

Specifies the OAuth2 client ID. This option is required when using `--auth oauth` for OAuth2 authentication.

**Usage:**
```shell
kodjin-cli --auth oauth --client-id <client-id>
```

### --client-secret <CLIENT_SECRET>

Specifies the OAuth2 client secret. This option is required when using `--auth oauth` for OAuth2 authentication.

> Note: Be cautious when using client secrets in command-line arguments, as they may be visible in command history or process listings.

### --scope <SCOPE>

Specifies OAuth2 scopes to request during authentication. This option is required when using `--auth oauth` for OAuth2 authentication.

**Usage:**
```shell
kodjin-cli --auth oauth --scope "<scope>"
```

**Example:**

Requesting a single scope:
```shell
$ kodjin-cli --auth oauth --token-url https://auth.example.com/token --client-id myclientid --client-secret myclientsecret --scope "user/Patient.read" install hl7.fhir.us.core@4.0.0
```

Requesting multiple scopes:
```shell
$ kodjin-cli --auth oauth --token-url https://auth.example.com/token --client-id myclientid --client-secret myclientsecret --scope "user/Patient.read user/Patient.write" install hl7.fhir.us.core@4.0.0
```

## --insecure-certificates

Skips TLS (Transport Layer Security) certificate validation, allowing connections to servers with self-signed or invalid certificates.

> Note: This option should be used with caution and only in trusted development or testing environments, as it bypasses a key security measure.

**Usage:**
```shell
kodjin-cli --insecure-certificates
```

**Example:**

In this example, we will install the specified Implementation Guide without verifying the server's TLS certificate.
```shell
$ kodjin-cli --insecure-certificates install hl7.fhir.us.core@4.0.0
```

## --request-timeout <REQUEST_TIMEOUT>

Limit the waiting time of the response to REQUEST_TIMEOUT (in seconds). The default value is 30 sec. If you want to wait more or less you can change this value by adding `--request-timeout` to the request.

**Usage:**
```shell
kodjin-cli --request-timeout <number>
```

**Example:**

In this example, we will increase waiting time for the response:
```shell
$ kodjin-cli --request-timeout 40 install hl7.fhir.us.core@4.0.0
```

## --errors-output <ERRORS_OUTPUT>

Specifies the output location for error logs generated when working with Implementation Guides (IGs). This option allows you to control where error messages are displayed or saved.

Available Options:

- `stderr` – processed files, Implementation Guides (IGs), OperationOutcomes are written directly to the console. This is the default value.
- `directory` – titles of IGs and files processed are written to the console, but OperationOutcomes are saved in newline-delimited JSON (.ndjson) files within the default directory.

Each system has its own default directory:

| Platform | Value                                | Example                                        |
| -------- | ------------------------------------ | ---------------------------------------------- |
| Linux    | $XDS_DATA_HOME or $HOME/.local/share | /home/<username\>/.local/share                 |
| macOS    | $HOME/Library/Application Support    | /Users/<username\>/Library/Application Support |
| Windows  | {FOLDER_LocalAppData}                | C:\Users\\<username\>\AppData\Local            |

- `folder path` - instead of writing OperationOutcome .ndjson files to the default directory you can choose any directory

**Usage:**
```shell
kodjin-cli --errors-output=<stderr|directory|folder path> <command>
```

**Examples:**

As `stderr` is a default value to use this value we do not need to add an option:

```shell
$ kodjin-cli install hl7.fhir.au.base@4.2.2-ballot
```

The next example is to write outcome files to the directory:
```shell
$ kodjin-cli --errors-output=directory install hl7.fhir.au.base@4.2.2-ballot
```

Current example is how to write outcome files to the local directory. We will write them to the current directory:
```shell
$ kodjin-cli --errors-output=. install hl7.fhir.au.base@4.2.2-ballot
```

## -h, --help

Displays help information for kodjin-cli, including available commands, options, and usage examples.

- This option can be used with any command to get more details on its usage.
- Running kodjin-cli without arguments may also display the help menu.
- The [`kodjin-cli help` command](./command-help.md) also returns the help information.

Help options:

- `-h` - returns summary information
- `--help` - returns full help information

**Usage:**
```shell
kodjin-cli --help
```
or
```shell
kodjin-cli <command> --help
```

<b>Examples:</b>

Example for summary:
```shell
kodjin-cli -h
```
<details>
<summary>Response example for summary</summary>

  ```shell
  Kodjin management CLI

  Usage: kodjin-cli [OPTIONS] <COMMAND>

  Commands:
    server                Manage FHIR server URLs
    metadata              Show FHIR server metadata
    install               Install a FHIR package
    uninstall             Uninstall a FHIR package
    check                 Check if a FHIR package is installed
    tree                  Print dependency tree of a FHIR package
    info                  Show information about a FHIR package
    download              Download a package locally
    generate-completions  Generate command autocompletions
    help                  Print this message or the help of the given subcommand(s)

  Options:
    -s, --server <SERVER>
            Select which FHIR server to use. The default one will be used if not specified
        --insecure-certificates
            Skip TLS certificate validation
        --request-timeout <REQUEST_TIMEOUT>
            Timeout for requests (in seconds) [default: 30]
        --errors-output <ERRORS_OUTPUT>
            Where errors should be written to [default: stderr] [possible values: stderr, directory]
    -h, --help
            Print help (see more with '--help')
    -V, --version
            Print version
  ```

</details>

Example for full help information: 

```shell
kodjin-cli --help
```
<details>
<summary>Respone for for full help information: </summary>
  ```shell
  Kodjin management CLI

  Usage examples:
  $ kodjin-cli server add https://demo.kodjin.com/fhir
  $ kodjin-cli metadata
  $ kodjin-cli info de.gematik.epa
  $ kodjin-cli install hl7.fhir.us.core@4.0.0
  $ kodjin-cli --errors-output=directory install hl7.fhir.us.core@4.0.0
  $ kodjin-cli --server=kodjin-demo check hl7.fhir.us.core@4.0.0

  For full information, see --help for each subcommand.

  Usage: kodjin-cli [OPTIONS] <COMMAND>

  Commands:
    server                Manage FHIR server URLs
    metadata              Show FHIR server metadata
    install               Install a FHIR package
    uninstall             Uninstall a FHIR package
    check                 Check if a FHIR package is installed
    tree                  Print dependency tree of a FHIR package
    info                  Show information about a FHIR package
    download              Download a package locally
    generate-completions  Generate command autocompletions
    help                  Print this message or the help of the given subcommand(s)

  Options:
    -s, --server <SERVER>
            Select which FHIR server to use. The default one will be used if not specified

        --insecure-certificates
            Skip TLS certificate validation

        --request-timeout <REQUEST_TIMEOUT>
            Timeout for requests (in seconds)

            [default: 30]

        --errors-output <ERRORS_OUTPUT>
            Where errors should be written to

            [default: stderr]
            [possible values: stderr, directory]

    -h, --help
            Print help (see a summary with '-h')

    -V, --version
            Print version
  ```
</details>

## -V, --version

Dislays the current version of Kodjin CLI, that is installed locally

**Usage: **
```shell
kodjin-cli --version
```

**Examples:**

```shell
$ kodjin-cli --version
kodjin-cli 0.1.0
```
