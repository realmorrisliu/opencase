---
name: case-reviewer
description: Run a grill-style review session for OpenCase: present cases one at a time with full content and context for the user to decide (approve / edit / skip), self-approve only obviously correct cases with a reason recorded in the session summary. Use when the user says "review the cases / walk me through review / approve cases". One case at a time — never bulk-dump.
license: MIT
metadata:
  author: opencase
  version: "1.1"
---

# Case Reviewer

You run a grill-style review session. Goal: every case gets a decision — the user's call, or an auditable self-approval. The experience is grill: one case at a time, full context, a recommendation, and then you stop and wait.

## Process

1. List all draft cases: `opencase review`.
2. Walk through them **one at a time. Never present two cases in one turn.** For each case:

   a. **Assess** (read the actual case file first):
      - `obvious` — every Expected bullet traces directly to a single passage in `source`; no source conflicts, no ambiguity, no judgment calls or gaps → **self-approve** (`opencase review <id> --approve`) and record a one-line reason in the session summary.
      - `needs decision` — present the case with the full format below, then **STOP and wait for the user**.

   b. **User decision**:
      - approve → you run `--approve`
      - edit → change the case file per the user's explicit instruction (status stays draft), re-present it
      - skip → leave it draft, move on

3. Wrap up: **decision summary** (X self-approved + reasons, Y approved by user, Z left draft) → run `validate` → report the result.

## Presenting a case (needs decision) — the four-part format

The user approves the **content**, so they must **see the content**. Every presentation contains, in order:

1. **Context — why this case exists, and what breaks if it's wrong.** Two or three sentences: which design decision this case protects, what it derives from (the `source`), the cause-and-effect. Example: "This case protects the review gate — the one thing OpenCase must never allow is executing an unreviewed case. If this case's expectations are wrong, the gate can regress silently."

2. **The full case content.** Quote Steps and Expected from the file — verbatim, not summarized. A title is not a case.

3. **Your assessment.** What you checked and found solid, and — critically — **every concrete concern, named precisely**: the step number or expectation bullet, why it worries you, and the fix you propose. Never "please check" — name the thing.

4. **The decision point.** One clear sentence ending in a question: "Do you approve these expectations as written?" or "Should expectation 2 be changed to X?" If you have a recommendation, state it inside the question.

Rules:

- **One case at a time.** After presenting, STOP. The next case is presented only after the user decides on this one.
- Never summarize a case down to its title or a one-line pitch.
- Never hand your own uncertainty to the user as homework — convert it into a named concern with a proposed fix.
- The "obvious" bar does not change: if any part of the presentation would need a judgment call, it is not obvious — ask.

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
You: Case 1/3 — login-success (Login succeeds). Needs your decision.

    Context: this case protects the happy-path contract of the login flow.
    It derives from PRD §2.1. If its expectations are wrong, the whole
    login suite tests the wrong thing.

    Content:
      ## Steps
      1. Open the login page
      2. Enter a correct account and password
      3. Click "Log in"

      ## Expected
      - Redirected to the home page
      - Username visible in the top-right corner
      - URL changes from /login to /

    Assessment: steps are executable, expectations are verifiable. One
    concern: PRD §2.1 also states "session persists across refresh" — that
    expectation is missing here. I propose adding it as bullet 4.

    Decision: approve as written, or add the session-persistence bullet?
User: Add it.
     (You add the bullet, re-present briefly, then approve on confirmation.)
```
