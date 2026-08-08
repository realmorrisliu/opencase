---
id: login-wrong-password
title: Login fails (wrong password)
status: draft
mode: manual
source: Feishu PRD "User Center" v3 §2.2 (token: abc123)
---

## Steps

1. Open the login page
2. Enter a correct account and a wrong password
3. Click "Log in"

## Expected

- Stays on the login page
- Shows the error message "Incorrect account or password"
- No session is created (still logged out after refresh)
