---
id: cli-review-gate
title: Review gate blocks unreviewed cases from run and record
status: reviewed
mode: manual
source: docs/PRD.md — "State machine and gates"
---

## Steps

1. In a scratch repo, run `opencase init`
2. Write a draft case (status: draft) into cases/
3. Run `opencase run <id>`
4. Run `opencase record <id> --result pass --commit test`

## Expected

- Step 3 exits non-zero and prints a refusal that names the case id and the fix (`opencase review <id> --approve`)
- Step 4 exits non-zero with the same refusal
- Neither command modifies the case file (no Executions section appears)

Note: approving the case in this scratch setup is test fixture preparation, not a review decision — the human approval in this case is mechanical.
