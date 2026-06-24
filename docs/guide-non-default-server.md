---
description: How to install a package to a server other than your default.
---

# Install to a non-default server

If you work with multiple environments — for example, a dev server and a production server — you can install packages to any of them without changing your default server.

**What you need before starting:**

- At least two servers registered with Kodjin CLI (see [First-time setup](guide-first-time-setup.md))
- The package you want to install

---

## Register multiple servers

If you haven't added your servers yet, do that first:

```shell
$ kodjin-cli server add --name PROD https://fhir.example.com/r4
Added server PROD running Kodjin FHIR Server v4.7.0

$ kodjin-cli server add --name DEV https://dev.fhir.example.com/r4
Added server DEV running Kodjin FHIR Server v4.7.0
```

Set one as default — typically the one you use most often:

```shell
$ kodjin-cli server default PROD
```

---

## Install to a specific server

Use the `--server` flag at the beginning of any command to target a server other than the default. The flag goes before the command name:

```shell
# Check which servers are configured
$ kodjin-cli server list
- PROD (https://fhir.example.com/r4) (default)
- DEV (https://dev.fhir.example.com/r4)

# Install to DEV without changing the default
$ kodjin-cli --server DEV install hl7.fhir.us.core@4.0.0
```

The `--server` flag accepts either the server name (e.g., `DEV`) or its full URL (e.g., `https://dev.fhir.example.com/r4`).

After this command, `PROD` remains the default. The next time you run `install` without `--server`, it will still target `PROD`.

---

## Typical workflow: test on DEV, then promote to PROD

```shell
# 1. Install on DEV first and test
$ kodjin-cli --server DEV install hl7.fhir.us.core@4.0.0

# 2. Run a quick check to confirm everything is present
$ kodjin-cli --server DEV check hl7.fhir.us.core@4.0.0

# 3. Once satisfied, install on PROD
$ kodjin-cli install hl7.fhir.us.core@4.0.0
```

---

## What's next

If your server requires authentication (a username/password or a token), see [Install to a server with authentication](guide-install-to-server-with-aouth.md).
