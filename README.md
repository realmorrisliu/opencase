# OpenCase

Declarative test-case management: test cases are git-managed declarative files (`cases/*.md`); agents decompose and execute, **humans own the review gate**. Zero database, zero runtime dependencies, one Rust binary.

[![CI](https://github.com/realmorrisliu/opencase/actions/workflows/ci.yml/badge.svg)](https://github.com/realmorrisliu/opencase/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/realmorrisliu/opencase)](https://github.com/realmorrisliu/opencase/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Chinese version: [README.zh-CN.md](README.zh-CN.md)

## Table of contents

- [Why it exists](#why-it-exists)
- [Core concepts](#core-concepts)
- [State machine](#state-machine)
- [Install](#install)
- [Quick start](#quick-start)
- [Case format](#case-format)
- [Command reference](#command-reference)
- [Agent integration](#agent-integration)
- [Contributing](#contributing)
- [Development](#development)
- [License](#license)

## Why it exists

| | Test-management platforms (TestRail-style) | Automation frameworks (Playwright-style) | OpenCase |
|---|---|---|---|
| Declarative case layer | Yes, but in a database, detached from code | No | Yes, git files that live with the code |
| Execution | By humans | Yes | Hybrid: manual to agents, scripted to CI |
| Sign-off | Messy | None | Execution records with date + commit |
| Cases vs scripts | Drift | No such concept | `covered-by` link + validation |

## Core concepts

- **Purely declarative**: case files are the single source of truth. Review happens through git diffs.
- **Review gate**: `draft → reviewed` is driven by a grill-style review session (case-reviewer) — cases are presented one at a time for you to decide; obviously correct cases (every expectation traces to one source passage, no conflicts, no ambiguity) may be self-approved by the agent with a reason recorded in the session summary, everything else waits for your call. Unreviewed cases cannot execute; any edit resets a case to draft.
- **Hybrid execution**: `manual` mode is executed by an agent playing the manual test engineer (the CLI prints the prompt, the agent operates the app with browser tools, results land as dated records); `scripted` results belong to CI — OpenCase only verifies the `covered-by` link does not lie.
- **Mandatory failure attribution**: failed records must be classified `product-bug` / `test-bug` / `environment`. No fuzz.
- **Drift-hunting**: re-run a scripted case manually and produce an "observed vs expected" diff — catching what scripts can't see.
- **Drift hint**: scriptify records a hash of Steps/Expected (`drift-sha`); `review` compares and warns when the case changed since scriptify, so the covered-by script can't silently go stale.

## State machine

```
draft ──review --approve (review session)──▶ reviewed
  ▲                                            │
  └───────────review --edit (any edit)─────────┘
                        │
                        ▼
        run / record / scriptify gate: reviewed
        record additionally requires: mode=manual
```

## Install

Pre-built binaries for macOS (Apple Silicon), Linux (x86_64) and Windows (x86_64) are attached to every [release](https://github.com/realmorrisliu/opencase/releases). No Rust toolchain required.

**One-liner (macOS / Linux):**

```bash
curl -fsSL https://raw.githubusercontent.com/realmorrisliu/opencase/main/install.sh | sh
```

Pin a version: `VERSION=v0.1.1 sh install.sh`. The script installs to `/usr/local/bin` (asks for sudo if needed). Windows users download the `.exe` from the releases page and put it on PATH.

**From source (requires Rust):**

```bash
cargo install opencase
```

Then, in any project repo:

```bash
opencase init        # scaffolds cases/ with an example case
opencase validate    # check all cases
```

## Quick start

```bash
opencase init
opencase validate                  # 1 case(s), 0 problem(s)
opencase review                    # list draft cases
opencase review <id> --approve     # approval (driven by the review session)
opencase run <id>                  # execution prompt for the agent
opencase record <id> --result pass
opencase report                    # status / coverage / latest runs
```

The repo also ships two example cases (`cases/`) and script stubs (`tests/`); `validate` should report `2 case(s), 0 problem(s)`.

## Case format

```markdown
---
id: login-success              # required, globally unique
title: Login succeeds          # required
status: draft                   # draft | reviewed
mode: manual                    # manual | scripted
source: Feishu PRD §2.1 (token: xxx)   # required, source provenance
covered-by: tests/login.spec.ts # optional; required for scripted
drift-sha: 0c3e6a...           # set by scriptify; review compares Steps/Expected against it
---

## Steps

1. Open the login page
2. Enter a correct account and password
3. Click "Log in"

## Expected

- Redirected to the home page
- Username visible in the top-right corner

## Executions      ← maintained by the CLI, don't hand-edit

- 2026-08-08 | abc1234 | pass
- 2026-08-09 | def5678 | fail | product-bug | button unresponsive, filed issue #12
```

- Frontmatter is deliberately flat: `key: value`, values contain no newlines or `|`
- Record lines: `- YYYY-MM-DD | <commit> | pass|fail [ | category ] [ | note ]`; failed lines must carry attribution
- `covered-by` resolves against the repo root (e.g. `tests/login.spec.ts`); validate checks the file exists

## Command reference

| Command | What it does | Gate |
|---|---|---|
| `init` | scaffold a `cases/` directory with an example case | — |
| `validate` | schema, state machine, record lines, covered-by | — |
| `review [id]` | list drafts + drift warnings; `--approve` approve; `--edit` open editor (reviewed → draft after edit). Driven by the case-reviewer session, not meant for humans to type | approval decided in the review session |
| `run <id>` | print execution prompt (Steps + Expected + source) | reviewed |
| `record <id> --result pass\|fail [--category] [--commit] [--note]` | append a dated execution record; fail requires attribution; commit defaults to git HEAD | reviewed + manual |
| `report` | markdown report: status, mode, coverage, latest run, draft list | — |
| `scriptify <id> [--covered-by <path>] [--rebaseline]` | print conversion context (Steps + records), flip case to scripted; `--rebaseline` refreshes the drift baseline after the script was updated | reviewed + manual |

Global `--cases <dir>` selects the cases directory (default `cases/`).

## Agent integration

The agent skills are the other half of OpenCase — they are what actually writes, reviews and executes cases. They follow the [Agent Skills open standard](https://agentskills.io/specification) and are embedded in the binary, so install them with the CLI (no repo clone needed):

```bash
opencase skill install                 # default: pi (~/.agents/skills)
opencase skill install --agent claude  # or: codex, project (.agents/skills in cwd)
opencase skill install --force         # overwrite local edits
```

If your harness has its own skill installer (e.g. `npx skills add realmorrisliu/opencase` or `gh skill install realmorrisliu/opencase`), pointing it at this repo works too — the skills live in `skills/` with a `SKILL.md` each.

- **case-writer**: decompose multi-source requirements (Feishu PRDs, spec docs, grill/to-prd artifacts) into happy paths / edges / failure flows, write cases with provenance, never approve
- **case-reviewer**: grill-style review session — one case at a time, approve / edit / skip; self-approves only obviously correct cases with a recorded reason; no session and no self-approval when the user is absent
- **case-executor**: run → execute for real → judge against Expected → record with attribution; draft `gh issue` on product-bug; uncertain attribution defaults to test-bug and asks; includes the drift-hunting flow

The typical loop:

```
case-writer  writes cases from a PRD → case-reviewer runs the review session (you decide)
→ case-executor executes and records → report shows status → scriptify moves stable cases to CI
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) — design decisions live in [`docs/PRD.md`](docs/PRD.md); read it before touching the format, the state machine, or the gates. The tool is deliberately thin; prefer the smallest change that works. Report security issues privately per [SECURITY.md](SECURITY.md). Community standards: [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Development

```bash
cargo test    # unit tests + end-to-end smoke (tests/smoke.rs drives the real binary through the full loop)
```

## License

MIT — see [LICENSE](LICENSE).

---

[CONTRIBUTING.md](CONTRIBUTING.md) · [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) · [SECURITY.md](SECURITY.md)
