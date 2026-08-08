---
id: login-success
title: Login succeeds (happy path)
status: draft
mode: manual
source: Feishu PRD "User Center" v3 §2.1 (token: abc123)
covered-by: tests/login.spec.ts
---

## Steps

1. Open the login page
2. Enter a correct account and password
3. Click "Log in"

## Expected

- Redirected to the home page
- Username visible in the top-right corner
- URL changes from /login to /
