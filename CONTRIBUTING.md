# Contributing to OpenCase

Thanks for considering a contribution. The project is deliberately thin — the smallest change that works wins.

## Before you start

- Read [`docs/PRD.md`](docs/PRD.md) — it is where all design decisions live. Anything touching the case format contract, the state machine, or the gates must stay consistent with it.
- Open an issue first for anything that changes behavior. Small fixes can go straight to a PR.

## Development setup

```bash
cargo build
cargo test           # unit tests + end-to-end smoke (drives the real binary)
cargo clippy -- -D warnings
cargo fmt --check
```

The end-to-end smoke test (`tests/smoke.rs`) drives the real binary through the full loop on a copy of `cases/`.

## Making changes

1. Branch from `main` (`git checkout -b fix/...`)
2. Make the change — prefer the smallest diff that works
3. Run the full check suite above
4. Update docs (`README.md` / `README.zh-CN.md` / `docs/PRD.md`) if behavior changed
5. Open a PR; CI must be green

## Reporting security issues

See [SECURITY.md](SECURITY.md) — report privately, never in a public issue.

## Code of conduct

Be respectful and constructive — see [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
