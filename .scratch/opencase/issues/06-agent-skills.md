Status: ready-for-agent

# Issue 06: agent skills (case-writer / case-reviewer / case-executor)

## Parent

[PRD](../PRD.md) — Implementation Decisions "Agent integration"

## What to build

Three tool-agnostic SKILL.md files under `skills/` (with name/description frontmatter). `case-writer`: collect multi-source requirements (Feishu docs, Openspec specs, grill/to-prd artifacts, pasted text) → decompose user journeys / happy paths / edges → write case files per the contract with provenance → run validate → stop; never approves anything. `case-reviewer`: grill-style review session — list drafts, present one at a time with an assessment, self-approve only obviously correct cases (every expectation traces to one source passage, no conflicts, no ambiguity) with a one-line reason in the session summary, stop and wait for the user otherwise; no user present → no session, no self-approval. `case-executor`: run → execute for real with browser/computer-use tools → judge against Expected → record with attribution (uncertain attribution defaults to test-bug and asks the user) → draft gh issue on product-bug and write the issue number into --note → document the drift-hunting flow (scripted cases: no record, diff report only).

## Acceptance criteria

- [ ] case-writer covers multi-source decomposition and provenance requirements
- [ ] case-writer never approves cases (approval belongs to the review session, including obvious cases)
- [ ] case-reviewer presents one case at a time and waits for user decisions
- [ ] case-reviewer defines the "obvious" bar and requires a recorded reason for self-approvals
- [ ] case-reviewer refuses to run or self-approve when the user is absent
- [ ] case-executor covers the run → execute → judge → record chain
- [ ] case-executor covers uncertain attribution (default test-bug + ask) and gh issue drafting with the issue number in --note
- [ ] case-executor documents the drift-hunting flow
- [ ] All three skills have valid SKILL.md frontmatter

## Blocked by

- [01](01-case-format-and-validate.md)
- [03](03-run-and-record.md)
