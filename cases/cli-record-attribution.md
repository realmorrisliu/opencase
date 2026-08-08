---
id: cli-record-attribution
title: Failed records require attribution; pass forbids it
status: reviewed
mode: manual
source: docs/PRD.md — "Failure attribution"
---

## Steps

1. In a scratch repo, create and approve a manual case (fixture setup)
2. Run `opencase record <id> --result fail`
3. Run `opencase record <id> --result fail --category product-bug --commit abc --note "button unresponsive"`
4. Run `opencase record <id> --result pass --category test-bug --commit abc`
5. Run `opencase record <id> --result pass --commit abc`

## Expected

- Step 2 exits non-zero, message mentions `--category` and the three categories
- Step 3 succeeds; the case file gains an `## Executions` line containing date, `abc`, `fail`, `product-bug` and the note
- Step 4 exits non-zero with a message explaining that --category applies only to failed runs
- Step 5 succeeds; the file now has exactly two record lines, in append order
- `opencase validate` reports 0 problems afterwards
