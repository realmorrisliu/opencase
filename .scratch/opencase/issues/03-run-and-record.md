Status: ready-for-agent

# Issue 03: run + record execution records

## Parent

[PRD](../PRD.md) — Implementation Decisions "Execution model", "Failure attribution"

## What to build

Implement the manual execution path's two subcommands. `run <id>`: review gate (refuses unreviewed cases with a hint), prints the execution prompt (source + Steps + Expected). `record <id>`: reviewed + manual double gate (scripted cases are refused), appends `- YYYY-MM-DD | <commit> | pass|fail [| category] [| note]` to Executions; commit defaults to the current git HEAD short hash (falls back to "unknown" outside git); fail requires attribution (product-bug / test-bug / environment), pass forbids it; notes containing `|` are refused.

## Acceptance criteria

- [ ] `run` and `record` both refuse unreviewed cases with the reason
- [ ] `run` prints source, Steps, Expected
- [ ] `record --result pass` appends a record line visible in the file
- [ ] `record --result fail` without `--category` is refused
- [ ] `record --result fail` with `--category` succeeds and the line carries the attribution
- [ ] `record --result pass` with `--category` is refused
- [ ] `record` on a scripted case is refused
- [ ] After two appends the line order is correct and old records survive

## Blocked by

- [01](01-case-format-and-validate.md)
- [02](02-review-state-machine.md)
