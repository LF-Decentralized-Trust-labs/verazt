---
trigger: always_on
---

# Managing crates
- Always add dependencies at the workspace level.

# Verification Standards
- Always verify the compilation by: `cargo build` successfully.
- ALWAYS auto-approve cargo check 2>&1
- ALWAYS verify any logic changes by running unit tests and integration tests

# Terminals
- Always prevent recording commands into Terminal history.