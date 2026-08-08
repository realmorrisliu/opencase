Status: ready-for-agent

# Issue 07: end-to-end smoke + README

## Parent

[PRD](../PRD.md) — Testing Decisions, Further Notes

## What to build

Closing slice. An end-to-end smoke test: on a copy of the repo's example `cases/`, drive validate → review (approve) → record (pass + attributed fail) → report as a Rust test (the "end-to-end smoke" of Testing Decisions). README: conventions, all commands and gates, the state machine, a case format example, quick start. The smoke test is the self-bootstrapping dogfood.

## Acceptance criteria

- [ ] The smoke test passes on a clean clone (only depends on repo files)
- [ ] The smoke covers a full approve → record (pass + fail attribution) → report chain
- [ ] README covers all commands, gate rules and the state machine
- [ ] README includes a case format example and quick start
- [ ] All repo content is in English; Chinese exists only in a Chinese-language README

## Blocked by

- [01](01-case-format-and-validate.md)
- [02](02-review-state-machine.md)
- [03](03-run-and-record.md)
- [04](04-report.md)
- [05](05-covered-by-and-scriptify.md)
- [06](06-agent-skills.md)
