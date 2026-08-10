---
name: opencase-release
description: "Prepare and publish a new OpenCase release: bump Cargo.toml version, update CHANGELOG.md, PR-first into protected main, then tag v<version> — the release workflow verifies, publishes to crates.io, and builds the platform binaries. Use when asked to cut a release, bump the version, publish to crates.io, or update the changelog."
---

# OpenCase Release

## Unified Publish Policy

GitHub Actions is the single publish path. **Do not run local `cargo publish`** — emergency-only fallback. Publishing happens in the `release` workflow, which requires the `CARGO_REGISTRY_TOKEN` secret (set via `gh secret set CARGO_REGISTRY_TOKEN`).

## Repository Policy

- `main` is protected: required checks + 1 review. Do not push directly.
- Push release prep to a branch, open a PR, merge after checks pass.

## Release inputs

- Target version (e.g. `0.2.0`)
- Release date `YYYY-MM-DD`
- Summary bullets grouped by `Added` / `Changed` / `Fixed` (from `CHANGELOG.md`)

## Versioned files (update together, in order)

1. `Cargo.toml` — `version = "<x.y.z>"`
2. `CHANGELOG.md`:
   - Move finalized items from `## [Unreleased]` into a new `## [<version>] - <date>` section
   - Keep `## [Unreleased]` at the top for the next cycle
3. README installation snippets if they pin a version (`VERSION=v<version>` examples)

## Validate before commit

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt --check
cargo package --locked   # or --allow-dirty before commit
```

## Commit, tag, publish

1. Branch → PR → merge to `main` (checks + review must pass)
2. Fast-forward local `main` to the merged commit
3. `git tag v<version>` + `git push origin v<version>`
4. The `release` workflow then: verifies tag matches Cargo.toml → builds the platform binaries + SHA-256 checksums → publishes to crates.io → creates the GitHub release

## Quick checklist

- [ ] `Cargo.toml` version bumped
- [ ] `CHANGELOG.md` new version section created
- [ ] local checks passed
- [ ] `cargo package --locked` passed
- [ ] release prep merged to `main`
- [ ] tag `v<version>` pushed
- [ ] `release` workflow green (verify + build + publish + release)
- [ ] crates.io shows the new version
- [ ] install.sh installs the new version (`VERSION=v<version> sh install.sh` in a temp dir)
