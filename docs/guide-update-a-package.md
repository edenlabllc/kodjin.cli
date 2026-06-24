---
description: How to update a FHIR Implementation Guide that's already installed on your server.
---

# Update a package

When a new version of an Implementation Guide is released, or when you want to re-install a package with changes, you need to tell Kodjin CLI what to do with resources that already exist on the server. By default it leaves them untouched — but you can change that behaviour.

**What you need before starting:**

- The package already installed (see [Install an Implementation Guide](guide-install-an-IG.md))
- The updated package name and version you want to install

---

## Why the default is "skip"

Kodjin CLI's default behaviour is to skip any resource that's already on the server. This is a safe default — it prevents accidental overwrites and makes it safe to re-run `install` without worrying about breaking anything that's already working.

```shell
# Re-running install does nothing to existing resources
$ kodjin-cli install hl7.fhir.us.core@4.0.0
```

---

## Update only resources that have changed

The `sync` option compares each resource in the package to what's on the server. It only uploads a resource if the content is different. This is the safest way to apply updates — nothing changes unless it needs to.

```shell
$ kodjin-cli install hl7.fhir.us.core@4.0.0 --existing-resources sync
```

Use this when upgrading to a new patch version, or when you know the package has changed and want the server to reflect those changes.

---

## Replace all resources unconditionally

The `overwrite` option replaces every resource regardless of whether it has changed. Every resource in the package gets re-uploaded.

```shell
$ kodjin-cli install hl7.fhir.us.core@4.0.0 --existing-resources overwrite
```

Use this when you want a guaranteed clean state — for example, after a manual edit on the server that you want to undo. Be careful with this option on a production server, as it will overwrite resources that may be in active use.

---

## Upgrade to a new version

To move from one version of a package to another, simply install the new version. Kodjin CLI treats different versions as separate packages — resources from the old version remain on the server and are not removed automatically.

```shell
# Install the newer version alongside the existing one
$ kodjin-cli install hl7.fhir.us.core@5.0.1
```

If you want to remove the old version afterwards, use 
```shell
kodjin-cli uninstall hl7.fhir.us.core@4.0.0
```
