---
id: cli-report
title: report renders status, coverage and latest runs as markdown
status: reviewed
mode: manual
source: docs/PRD.md — "Reporting"
---

## Steps

1. In a scratch repo, create two cases: one reviewed+manual with a pass record, one draft
2. Run `opencase report`

## Expected

- Output is valid markdown: `# Opencase Report` heading, a counts line (Total/reviewed/draft), a coverage line (Manual/scripted/automated coverage), and a table with header separator
- The reviewed case's row shows `pass` and the record date; the draft case's row shows `—` for last run
- The draft case appears in the `## Draft (need review)` section
- The counts line reads exactly `Total: 2 | reviewed: 1 | draft: 1` and the coverage line reads `Manual: 2 | scripted: 0 | automated coverage: 0`

## Executions

- 2026-08-08 | 3caae2e | pass | executed in scratch repo, all expectations observed
