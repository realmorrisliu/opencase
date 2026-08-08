Status: ready-for-agent

# PRD: OpenCase — Declarative Test-Case Management

## Problem Statement

Most of a junior/mid-level test engineer's work — test-case decomposition, manual testing, part of automation development — can be replaced or assisted by an agent workflow. But existing alternatives fall between two stools: test-management platforms (TestRail-style) are heavy, their cases drift from the real system, and execution depends on people; while pure automation frameworks (Playwright-style) have an execution layer but no declarative layer — what is covered, why it is tested, and who signed off on it are nowhere in sight. No tool combines "declarative case files + agent execution + human review gate + reporting".

## Solution

OpenCase: an open-source convention + Rust CLI.

- **Purely declarative**: test cases are `cases/<id>.md` files — frontmatter carries metadata (source provenance, status, mode, script coverage), the body describes the case with `## Steps` / `## Expected`. Git is the single source of truth; review happens through diffs.
- **Review gate driven by a grill-style session**: state machine `draft → reviewed → executable`. The case-reviewer session presents cases one at a time for the human to decide; obviously correct cases (every expectation traces to a single source passage, no conflicts, no ambiguity) may be self-approved by the agent with a reason recorded in the session summary. Unreviewed cases cannot execute; any edit resets a case to draft.
- **Hybrid execution model**: `manual` mode is executed by an agent playing the manual test engineer (the CLI prints the prompt, the agent operates the app with browser tools, results land as dated execution records); `scripted` mode belongs to CI — OpenCase only verifies the `covered-by` link does not lie.
- **Mandatory failure attribution**: failed runs must be classified (`product-bug` / `test-bug` / `environment`); issues are drafted on demand via `gh`.
- **Reporting**: status, mode, automation coverage, latest run — a markdown report a QA lead would read.

## User Stories

1. As a QA lead, I want agent-written test cases decomposed from PRD sources (Feishu docs, Openspec specs, grill/to-prd artifacts), so that case decomposition work is automated
2. As a QA lead, I want every case to record its source provenance, so that I can trace where each expectation came from
3. As a QA lead, I want a grill-style review session that walks me through unreviewed cases one at a time, so that I inject real-world expectations before anything executes
4. As a QA lead, I want obviously correct cases self-approved by the agent with a visible reason, so that my attention is not spent on decisions that need no judgment
5. As a QA lead, I want unreviewed cases blocked from execution, so that nothing runs against unverified expectations
6. As a QA lead, I want editing a case to reset it to draft, so that changed cases always get re-reviewed
7. As an agent, I want a run command that prints a case's steps and expected results, so that I can execute it as a manual tester
8. As an agent, I want to record execution results with date + commit ref, so that sign-off is traceable
9. As a QA lead, I want failed runs to require an attribution category, so that failures are triaged as product bug / test bug / environment issue rather than shrugged off
10. As a dev, I want a failed product-bug run to optionally draft a GitHub issue via `gh`, so that bugs don't get lost in case files
11. As a QA lead, I want a `covered-by` link between case and automation script, so that the report can show automated coverage
12. As a QA lead, I want validation that covered-by files exist, so that cases don't silently rot away from deleted scripts
13. As a QA lead, I want to convert a manual case into a scripted one, so that regression coverage moves to CI where it's fast and deterministic
14. As a QA lead, I want to re-run manual cases as drift-hunting sweeps, so that changes the scripts can't see get caught
15. As a QA lead, I want a report with status / mode / coverage / last-run summary, so that I can communicate test status without opening the files
16. As a maintainer, I want everything git-native with zero database, so that review happens via diff and the tool stays thin

## Implementation Decisions

### Shape

- **Rust, single crate, single binary**, zero runtime dependencies. CLI subcommands: `init` / `validate` / `review` / `run` / `record` / `report` / `scriptify`, global `--cases <dir>` (default `cases/`). `init` scaffolds a `cases/` directory with an example case so non-Rust users can start without cloning the repo.
- Dependencies minimized: frontmatter parsed by hand (the format is deliberately flat, see below), no YAML library; CLI arg parsing hand-rolled to keep zero deps — implementer's choice, not a contract.

### Case file format (contract)

```
---
id: login-success          # required, globally unique
title: Login succeeds      # required
status: draft              # draft | reviewed
mode: manual               # manual | scripted
source: Feishu PRD §2.1 (token: xxx)   # required, source provenance
covered-by: tests/login.spec.ts      # optional; required for scripted
drift-sha: 0c3e6a...                # set by scriptify; review compares Steps/Expected against it
---

## Steps

1. Open the login page
2. Enter a correct account and password
3. Click "Log in"

## Expected

- Redirected to the home page
- Username visible in the top-right corner

## Executions

- 2026-08-08 | commit abc1234 | pass
- 2026-08-09 | commit def5678 | fail | product-bug | button unresponsive, filed issue #12
```

- Frontmatter is deliberately **flat `key: value`**; values contain no newlines or `|`; the body has exactly three sections: `Steps` / `Expected` / `Executions` (the record area is CLI-maintained, appended at the end of the file).
- Execution record line format: `- YYYY-MM-DD | <commit> | pass|fail [ | category ] [ | note ]`. Failed lines must carry a category; on pass lines the first extra field is the note (field positions unambiguous by result).

### State machine and gates

- `draft → reviewed`: driven by the **case-reviewer session** (grill-style): cases are presented one at a time for the human to decide; obviously correct cases (Expected traces to a single source passage, no conflicts, no ambiguity) may be self-approved by the agent, with the reason recorded in the session summary; everything else waits for the human's call.
- `review --edit`: opens `$EDITOR`; if the case was reviewed, it resets to draft after the edit.
- Gates: `run` and `record` both require reviewed; `record` additionally requires mode=manual (scripted results belong to CI; opencase refuses to record them).

### Execution model

- **manual**: `run <id>` prints the execution prompt (Steps + Expected + source); the agent operates the app with its own browser/computer-use tools, then `record <id> --result pass|fail [--category ...] [--note ...]` lands the record. `--commit` defaults to the current git HEAD short hash.
- **scripted**: scripts live in `tests/` and run in CI; OpenCase does not touch their results. It does two things: `validate` checks that the `covered-by` file exists; the **drift hint** flags declaration-coverage drift — scriptify records a hash of the case's Steps/Expected (`drift-sha`), and `review` compares the current content against it, hinting "Steps/Expected changed since scriptify — script may be stale". `scriptify <id> --rebaseline` refreshes the baseline after the script has been updated. Case-side drift only; the reverse direction (script changed, case stale) is deliberately not detected — `ponytail:` it needs a script-content baseline recorded at validate time, add when it bites.
- **scriptify**: `scriptify <id>` prints the conversion context (Steps + execution records) for the agent to turn into a script, then updates the case to mode=scripted with `covered-by` written in. Regression = run scripted only; manual mode remains for new-case exploration, failure triage, and periodic drift-hunting (re-running a scripted case manually, producing an "observed vs expected" diff).

### Failure attribution

- `record --result fail` must carry `--category`, one of: `product-bug` / `test-bug` / `environment`. When attribution is uncertain, the agent defaults to test-bug and asks the user — never blames the product by default.
- No issue integration is built: the agent drafts with `gh issue create` (repro steps, observed vs expected, severity suggestion) and writes the issue number into `--note`.

### Reporting

- `report` outputs markdown: totals / reviewed / draft, manual / scripted / automation coverage (`covered-by` count), one row per case (status, mode, covered-by, latest run), plus the to-review draft list.

### Agent integration

- Three tool-agnostic SKILL.md files under `skills/`: `case-writer` (reads multi-source requirements → decomposes into user journeys/happy paths/edges → writes cases → runs validate; never approves), `case-reviewer` (grill-style review session: presents cases one at a time, self-approves only obviously correct cases with a recorded reason, never self-approves when the user is absent) and `case-executor` (run → execute for real → judge against Expected → record with attribution → file gh issue on demand; includes the drift-hunting flow).

### Not built

- Database / multi-user / permissions; agent execution harness (browser automation); CI integration; issue-system integration; cross-project case sharing.

## Testing Decisions

- A good test only exercises external behavior: the CLI subcommands' inputs, outputs and file side effects — never internals.
- What is tested: state-machine transitions (approve / edit reset / gate rejections), execution-record parsing and appending (including fail-requires-attribution), validate rules (missing fields, duplicate ids, covered-by missing, bad record lines), report output.
- Form: Rust unit tests (parsing and state-machine logic) + one end-to-end smoke test (drives the real binary through validate → review → record → report on a copy of the example `cases/`).
- Prior art: greenfield, no existing tests; the smoke test is the self-bootstrapping dogfood.

## Out of Scope

- Agent execution harness and browser automation (users bring their own agent + browser tools; `skills/` only provides workflow conventions)
- CI integration and scripted-result feedback
- Database, multi-user, permissions, audit login
- Issue-system integration (on-demand `gh` only)
- Code-coverage statistics (distinct from test coverage)
- Test-case marketplace / cross-organization sharing

## Further Notes

- Acceptance: walk a real PRD (e.g. a Feishu requirements doc) through the full loop — write cases → human review → agent execution → report — producing a report a QA lead would read.
- All design decisions here come from the 2025-08-08 grill session; this PRD is where they land.
- The first Python CLI draft was deleted (implementation language is Rust, see Implementation Decisions); `cases/` examples and `tests/` stubs remain as format examples.
