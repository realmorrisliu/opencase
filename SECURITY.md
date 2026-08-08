# Security Policy

## Reporting a vulnerability

Please report security issues **privately** — never in a public issue or PR:

- Use GitHub's private vulnerability reporting: the repository's **Security** tab → **Report a vulnerability**, or
- Email morrisliu1994@outlook.com

You should receive an acknowledgement within 3 business days. The fix timeline
depends on severity; we will keep you informed.

## Scope

- The `opencase` binary (Rust, zero dependencies — the attack surface is the
  files and directories you point it at)
- The three agent skills (`skills/`): note that skills are instructions your
  agent executes — review their content before installing, and audit updates
  after `opencase skill install --force`

## Out of scope

- Vulnerabilities in third-party agent harnesses or browser tools that OpenCase
  integrates with through its skills
