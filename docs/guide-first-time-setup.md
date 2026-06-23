---
description: Connect Kodjin CLI to your FHIR server for the first time.
---

# First-time setup

Before you can install any Implementation Guide, Kodjin CLI needs to know which FHIR server to talk to. This guide walks you through connecting to a server and confirming everything works.

**What you need before starting:**

- Kodjin CLI installed on your machine (see [Installation](installation.md))
- The URL of a running FHIR server (e.g., `https://demo.kodjin.com/fhir`)

---

## Step 1 — Register your server

Tell Kodjin CLI about your server by adding it to your local config. The `--name` flag gives it a short label you can use instead of typing the full URL every time.

```shell
$ kodjin-cli server add --name DEMO https://demo.kodjin.com/fhir

Added server DEMO running Kodjin FHIR Server production
```

If you see `Added server ...` in the response, the CLI successfully reached your server. If the command returns an error, double-check that the URL is correct and the server is running.

---

## Step 2 — Set it as your default

Kodjin CLI can store multiple servers (for example, separate dev, staging, and production environments). Setting one as the default means you don't need to specify it every time you run a command.

When you add a new server - it is automatically set as default. 

To see all registered servers and confirm which one is default:

```shell
$ kodjin-cli server list

List of currently configured servers:
- DEMO (https://demo.kodjin.com/fhir) (default)
```

In case you want to set as default previously added server:

```shell
$ kodjin-cli server default DEMO

Setting DEMO as the default server
```

---

## Step 3 — Verify the connection

Run `metadata` to fetch basic information from the server and confirm it's reachable:

```shell
$ kodjin-cli metadata

Name: Kodjin FHIR server 4.0.1 CapabilityStatement
Publisher: EdenLab
URL: https://demo.kodjin.com/fhir/metadata
Date: 2025-07-16T12:28:38.732186848+00:00
Software: Kodjin FHIR Server
Software Version: v5.2.0
FHIR Version: 4.0.1
```

This output confirms:

- The server responded successfully
- The FHIR version it supports (here `4.0.1`, also known as FHIR R4)
- The server software version

If `metadata` returns an error, check your network connection and whether the server URL is correct.

---

## What's next

Your server is registered and set as default. All subsequent commands — `install`, `check`, `uninstall` — will target it automatically.

Next step: [Look up a package before installing](guide-look-up-a-package.md)
