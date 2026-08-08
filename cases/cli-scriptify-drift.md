---
id: cli-scriptify-drift
title: scriptify flips a case to scripted and drift detection flags stale scripts
status: draft
mode: manual
source: docs/PRD.md — "Execution model"
---

## Steps

1. In a scratch repo, create and approve a manual case (fixture setup)
2. Run `opencase scriptify <id>` with the covered-by file present
3. Run `opencase review <id>`
4. Edit the case's Steps (add a step)
5. Run `opencase review <id>`
6. Run `opencase scriptify <id> --rebaseline`
7. Run `opencase review <id>` again

## Expected

- Step 2 prints Steps + execution records as conversion context, flips the case to `mode: scripted`, writes `covered-by` and a `drift-sha` baseline; `opencase validate` passes
- Step 3 shows no drift warning
- Step 5 shows the drift warning mentioning `--rebaseline`
- Step 7 shows no drift warning (baseline refreshed)
- `opencase record` on the scripted case is refused (scripted results belong to CI)
