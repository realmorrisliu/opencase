# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Fixed

- Pass records with a note were misparsed as carrying a category — field positions are now unambiguous by result
- Records could be silently appended into the Expected section of a case that mentions "## Executions" in its body — section headers are now matched as standalone lines

## [0.1.0] - 2026-08-08

### Added

- Declarative test-case format (`cases/*.md`): flat frontmatter (id/title/status/mode/source/covered-by/drift-sha), `## Steps` / `## Expected` / `## Executions` sections
- State machine with a human-owned review gate: `draft → reviewed → executable`, any edit resets to draft
- Hybrid execution model: `manual` (agent-executed, dated records with git commit refs) and `scripted` (CI-owned, linked via `covered-by`)
- Mandatory failure attribution: `product-bug` / `test-bug` / `environment`
- Drift detection: scriptify records a Steps/Expected hash; `review` warns when the case outlives its script
- Three agent skills (Agent Skills standard): case-writer, case-reviewer (grill-style review session), case-executor — embedded in the binary, installed via `opencase skill install`
- Zero-dependency Rust CLI: `init` / `validate` / `review` / `run` / `record` / `report` / `scriptify` / `skill`
- Install script (`install.sh`) with SHA-256 checksum verification; pre-built binaries for macOS/Linux/Windows on every release
- MIT license, dual-language READMEs, CI (test + clippy + fmt), release workflow

## [0.1.1] - 2026-08-08

### Fixed

- `opencase init` scaffolded `cases/cases` instead of `cases/` — now creates the target directory itself
- `install.sh` verified checksums against asset names that didn't match the downloaded file; pinned-version URLs now use the correct `releases/download/<tag>/` path

### Changed

- Release pipeline: dropped Intel Mac (darwin-x86_64) builds; added keyless build provenance attestations (`actions/attest`) to release binaries
- crates.io publish is idempotent (skips when the version already exists)

## [Unreleased]

### Added

- `report` now surfaces run-level detail: failures count in the summary line, a `## Failures (need triage)` drill-down (case, date, category, note), and a `## Recent executions` list (latest 10 records across cases)
