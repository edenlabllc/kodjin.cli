## Release v0.2.1
Sep 03, 2026

- Release pipeline aligned with RMK: create tags from `release/*` / `hotfix/*` merges, publish RC and stable GitHub releases, upload versioned and `latest` / `latest-rc` artifacts to S3
- Fail the release if `Cargo.toml` / `Cargo.lock` do not match the release tag
- Installer download now uses `curl --fail --location` with retries so failed downloads abort instead of installing a broken archive

## Release v0.2.0
Sep 03, 2026

- First public open-source release of Kodjin CLI
- Publish multi-version documentation with MkDocs and Mike (`latest` + version aliases)
- Add GitHub Actions workflow to validate docs on feature branches and publish versioned docs from master
- Expand user guides for installing Implementation Guides from packages, local directories, and Git repositories
- Bump package version to 0.2.0 and refresh GoReleaser configuration
