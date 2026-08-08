Status: ready-for-agent

# Issue 05: covered-by validation + scriptify

## Parent

[PRD](../PRD.md) — Implementation Decisions "Execution model", "State machine and gates"

## What to build

Full support for the case ↔ script link. `validate` gains two rules: the file a covered-by points at must exist; a scripted case must carry covered-by. Implement `scriptify <id>`: for a reviewed manual case, print the conversion context (Steps + execution records), then update the case to mode=scripted with covered-by written in; validate must pass afterwards. Scripted cases remain runnable via `run` (the drift-hunting path).

## Acceptance criteria

- [ ] `validate` errors when covered-by points at a nonexistent file
- [ ] `validate` errors when a scripted case lacks covered-by
- [ ] `scriptify` prints a conversion context containing Steps and execution records
- [ ] After `scriptify`, the case is scripted + covered-by and `validate` passes
- [ ] Scripted cases can still be `run` (not refused)
- [ ] `scriptify` refuses unreviewed cases

## Blocked by

- [01](01-case-format-and-validate.md)
- [03](03-run-and-record.md)
