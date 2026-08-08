Status: ready-for-agent

# Issue 01: Case format and parsing + validate

## Parent

[PRD](../PRD.md) — Implementation Decisions "Case file format (contract)"

## What to build

Implement the case file format contract for `cases/` and its parser, plus the `validate` subcommand. The parser covers the full PRD contract: flat frontmatter (id/title/status/mode/source/covered-by), the three body sections (Steps/Expected/Executions), and the execution record line format (failed lines must carry attribution). `validate` prints the full problem list and exits non-zero when there are problems.

## Acceptance criteria

- [ ] `validate` reports `0 problem(s)` for the repo's example cases
- [ ] Missing any of id/title/status/mode/source is an error
- [ ] Duplicate ids are an error
- [ ] Invalid status / mode values are errors
- [ ] Bad execution record lines (format errors, failed lines without attribution) are errors
- [ ] Every element of the PRD format contract has a corresponding check

## Blocked by

None - can start immediately
