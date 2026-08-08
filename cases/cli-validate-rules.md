---
id: cli-validate-rules
title: validate flags schema violations with actionable messages
status: draft
mode: manual
source: docs/PRD.md — "Case file format (contract)"
---

## Steps

1. In a scratch repo, write a case missing the `title` field
2. Run `opencase validate`
3. Write a case with `status: shipped` (invalid enum)
4. Run `opencase validate`
5. Write two cases with the same id
6. Run `opencase validate`

## Expected

- Step 2 reports `missing 'title'` naming the file, exits non-zero
- Step 4 reports `bad status 'shipped'` with the valid values listed, exits non-zero
- Step 6 reports `duplicate id` naming both files, exits non-zero
- A clean repo reports `N case(s), 0 problem(s)` and exits zero
