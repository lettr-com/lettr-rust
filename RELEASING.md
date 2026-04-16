# Releasing

This document describes how to cut a new release of `lettr`.

## Versioning

This project follows [Semantic Versioning 2.0.0](https://semver.org/).

- **MAJOR** — incompatible API changes (breaking changes to public types, method signatures, or behavior)
- **MINOR** — new functionality in a backwards-compatible manner (new endpoints, new optional fields, new builder methods)
- **PATCH** — backwards-compatible bug fixes

Pre-1.0 releases (`0.x.y`) may introduce breaking changes in minor bumps.

## Release Process

Releases are automated via GitHub Actions (`.github/workflows/publish.yml`). The workflow triggers on a **published GitHub Release** — not on tag push alone.

### 1. Prepare the release

1. Ensure `main` is green and up to date.
2. Bump the version in `Cargo.toml`:
   ```toml
   [package]
   version = "X.Y.Z"
   ```
3. Update `CHANGELOG.md`:
   - Move entries from `[Unreleased]` into a new `[X.Y.Z] - YYYY-MM-DD` section
   - Add a new empty `[Unreleased]` section at the top
   - Update the comparison links at the bottom of the file
4. Commit:
   ```sh
   git commit -am "release vX.Y.Z"
   git push
   ```

### 2. Create the GitHub Release

The tag must match the version with a `v` prefix (e.g. `v1.0.0` for version `1.0.0`).

Using the `gh` CLI:
```sh
gh release create vX.Y.Z --generate-notes --title "vX.Y.Z"
```

Or via the GitHub UI: **Releases → Draft a new release → Create tag `vX.Y.Z` on main → Publish release**.

Publishing the release triggers the `publish.yml` workflow, which:
1. Verifies `Cargo.toml` version matches the tag
2. Runs `cargo test`
3. Runs `cargo publish --dry-run`
4. Publishes to crates.io via `CARGO_REGISTRY_TOKEN`

### 3. Verify

- Check the Actions tab to confirm the workflow succeeded
- Confirm the new version appears on [crates.io/crates/lettr](https://crates.io/crates/lettr)
- Confirm docs built on [docs.rs/lettr](https://docs.rs/lettr)

## Prerequisites

The following GitHub Actions secrets must be configured (Settings → Secrets and variables → Actions):

- `CARGO_REGISTRY_TOKEN` — a crates.io API token with publish scope for the `lettr` crate.
  Create one at <https://crates.io/settings/tokens>.

## Breaking Changes Policy

Before 1.0:
- Breaking changes are allowed in minor releases (`0.x.0`)
- Document them clearly in `CHANGELOG.md` under a `### Changed` or `### Removed` heading, prefixed with `**BREAKING**:`

After 1.0:
- Breaking changes require a major version bump
- Deprecate symbols for at least one minor release before removing them where feasible

## Yanking a release

If a release has a critical bug, yank it from crates.io:
```sh
cargo yank --version X.Y.Z
```

Yanking does not delete the release — it prevents new projects from picking it up while allowing existing `Cargo.lock` files to continue resolving. Follow up with a patch release containing the fix.
