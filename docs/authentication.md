# Kodjin CLI Basic Authentication

## Overview

The Kodjin CLI now supports HTTP Basic Authentication through the `-H` (header) flag, allowing you to authenticate requests with username and password credentials.

## Basic Authentication Setup

### Step 1: Encode Credentials

Basic authentication requires your credentials to be base64 encoded in the format `username:password`.

```bash
echo -n 'user:password' | base64
```

**Important**: The `-n` flag prevents echo from adding a newline character, which would corrupt the base64 encoding.

**Example:**

```bash
echo -n 'john.doe:mypassword123' | base64
```

### Step 2: Add Authorization Header

Use the `-H` flag to add the Authorization header with your base64-encoded credentials:

```bash
kodjin-cli -H 'Authorization: Basic <base64-encoded-credentials>' <command>
```

## Security Considerations

### Environment Variables

For better security, store credentials in environment variables:

```bash
# Use in commands
ENCODED_CREDS=$(echo -n 'username:password' | base64)
kodjin-cli -H "Authorization: Basic $ENCODED_CREDS" metadata
```
