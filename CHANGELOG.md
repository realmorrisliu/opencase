# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

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
