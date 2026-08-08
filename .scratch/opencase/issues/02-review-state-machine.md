Status: ready-for-agent

# Issue 02: review state machine

## Parent

[PRD](../PRD.md) — Implementation Decisions "State machine and gates"

## What to build

Implement the `review` subcommand's state machine: with no id, list all draft cases; `--approve` flips draft → reviewed (the only status-flip path); `--edit` opens `$EDITOR` and resets a reviewed case to draft. Status changes are visible in `validate` output. (The grill-style review session that drives these commands is issue 06; this issue is the CLI mechanics.)

## Acceptance criteria

- [ ] `review` lists all draft cases (id / title / source)
- [ ] After `--approve`, the case shows reviewed in validate
- [ ] `--approve` on an already-reviewed case is a no-op
- [ ] `--edit` on a reviewed case resets it to draft
- [ ] `--edit` on a draft case leaves it draft
- [ ] A clear message when there are no draft cases

## Blocked by

- [01](01-case-format-and-validate.md)
