---
description: How to connect to a FHIR server that requires authentication.
---

# Install to a server with authentication

Many FHIR servers require credentials before accepting requests. Kodjin CLI supports three authentication methods: 

- Basic (username and password), 
- Bearer token, 
- OAuth2 client credentials.

You don't need to store credentials in your config — pass them directly on the command line each time. The `--auth` flag and the credential flags go before the command name.

---

## Basic authentication

Use a username and password. This is the simplest method, often used for internal or development servers.

```shell
$ kodjin-cli --auth basic --user admin --password secret install hl7.fhir.us.core@4.0.0
```

> **Note:** Passwords passed on the command line may appear in your shell history. On shared machines, consider using Bearer or OAuth2 instead.

---

## Bearer token

Use a pre-issued token (for example, a JWT). This is common when the server uses an identity provider that issues tokens separately.

```shell
$ kodjin-cli --auth bearer --bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9... install hl7.fhir.us.core@4.0.0
```

Replace `eyJhbGci...` with your actual token. The token is sent as an `Authorization: Bearer <token>` header on every request.

---

## OAuth2 (client credentials)

Use this when your server is protected by an OAuth2 authorization server. You'll need a client ID and client secret from your auth provider.

```shell
$ kodjin-cli --auth oauth \
  --token-url https://auth.demo.kodjin.com/token \
  --client-id myclientid \
  --client-secret mysecret \
  install hl7.fhir.us.core@4.0.0
```

What each flag means:

- `--token-url` — The URL of the OAuth2 token endpoint (provided by your auth server)
- `--client-id` git clientid \
  --client-secret mysecret \
  --scope "system/*.write" \
  install hl7.fhir.us.core@4.0.0
```

---

## Combining auth with other flags

Authentication flags work alongside all other Kodjin CLI flags. For example, to install to a non-default server with OAuth2:

```shell
$ kodjin-cli --server DEV \
  --auth oauth \
  --token-url https://auth.example.com/token \
  --client-id myclientid \
  --client-secret mysecret \
  install hl7.fhir.us.core@4.0.0
```
