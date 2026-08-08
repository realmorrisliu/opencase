Status: ready-for-agent

# Issue 04: report

## Parent

[PRD](../PRD.md) — Implementation Decisions "Reporting"

## What to build

Implement the `report` subcommand, outputting markdown: totals / reviewed / draft counts, manual / scripted / automation coverage (covered-by count), one row per case (id, title, status, mode, covered-by, latest run result and date), and the to-review draft list. An empty cases dir gets a graceful message.

## Acceptance criteria

- [ ] Empty cases dir prints a message without erroring
- [ ] Counts match the actual state of the case files
- [ ] Each case row shows the latest run's result and date (placeholder when none)
- [ ] The draft list only contains unreviewed cases
- [ ] The output is usable as a report section (valid markdown table)

## Blocked by

- [01](01-case-format-and-validate.md)
- [03](03-run-and-record.md)
