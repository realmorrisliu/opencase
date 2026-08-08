---
name: case-reviewer
description: Run a grill-style review session for OpenCase: present cases one at a time for the user to decide (approve / edit / skip), self-approve only obviously correct cases with a reason recorded in the session summary. Use when the user says "review the cases / walk me through review / approve cases". One case at a time, never bulk-dump.
license: MIT
metadata:
  author: opencase
  version: "1.0"
---

# Case Reviewer

You run a grill-style review session. Goal: every case gets a decision — the user's call, or an auditable self-approval. The experience is grill: one case at a time, with an assessment and a recommendation, waiting for feedback.

## Process

1. List all draft cases: `opencase review`.
2. Walk through them **one at a time. Never bulk-dump.** For each case:

   a. **Present**: id, title, source, Steps, Expected, plus your assessment.

   b. **Assess**:
      - `obvious` — every Expected bullet traces directly to a single passage in `source`; no source conflicts, no ambiguous terms, no judgment calls or gaps → **self-approve** (`opencase review <id> --approve`) and record a one-line reason in the session summary.
      - `needs decision` — list the concrete concerns (source conflict / vague expectation / missing edge case / coverage gap / case-script drift) → **STOP and wait for the user**.

   c. **User decision**:
      - approve → you run `--approve`
      - edit → change the case file per the user's explicit instruction (status stays draft), re-present it
      - skip → leave it draft, move on

3. Wrap up: **decision summary** (X self-approved + reasons, Y approved by user, Z left draft) → run `validate` → report the result.

## The "obvious" bar (all must hold to self-approve)

- Every Expected bullet traces to a single source passage; no conflicts between sources
- No ambiguity: Steps are executable, Expected is verifiable, no subjective words ("feels good", "reasonable" do not count)
- No gaps: no missing information that would require you to fill in the blanks
- If any point is uncertain → not obvious. Ask.

## Discipline

- Self-approvals must appear in the session summary with a one-line reason. Never silent.
- **No user present: do not run the session and do not self-approve.** Leave drafts alone.
- Never edit Expected on your own — only on the user's explicit instruction.
- An edited case resets to draft and needs a fresh decision; never re-approve it as a courtesy.

## Example exchange

```
You: Case 1/3 — login-success (Login succeeds)
     source: PRD §2.1
     Expected: redirect to home / username visible / URL changes
     Assessment: obvious — all three bullets come straight from §2.1,
     no conflicts, no ambiguity. Self-approving.
     ── next ──
     Case 2/3 — checkout-success (Checkout succeeds)
     Expected: order created / redirected to order page
     Assessment: ⚠️ needs your decision — PRD §3.1 says "returns an
     order number after submit", but this case's Expected omits it.
     Add it or keep as is?
User: Add it.
     (You edit the case, re-present, then approve on confirmation.)
```
