---
name: verify
description: >-
  Verify this application behaves correctly before shipping. Use when the user says
  "test the app", "is this ready to ship", "verify", "check for bugs", "QA this",
  or after completing a feature. Operates semantic actors, watches database/logs
  live, and returns replay-confirmed proof of bugs.
license: Apache-2.0
allowed-tools: [Bash, Read, Grep]
---

# Verify Skill

Verifies application behavior against independent invariants and physical hardware reality.

## The Loop
1. Analyze changed surfaces via Tree-Sitter AST.
2. Run scoped semantic actors across roles.
3. Assert declared invariants live with database and server watchers.
4. **Adaptive Stress-Testing & Algorithmic Ramps:**
   - The agent is NOT confined to a fixed profile. The agent can dynamically set and ramp budgets:
     - Test at 200MB RAM first; ramp to 500MB, 1GB, etc., while scaling concurrent requests.
     - Detect exact breaking points, memory creep, or $O(N^2)$ algorithm traps (e.g. nested loops instead of B-Trees or HashMaps).
5. If failures occur, compile to deterministic scripts and confirm reproduction (3/3).
6. **Negative Constraints & Integrity Rules:**
   - **Zero Test Tampering:** NEVER edit, comment out, or relax an invariant assertion to force a failing scenario to pass.
   - **Fix Source Only:** When a scenario fails, the finding must be handed to the coding agent to fix the *application source code*, never the test oracle.
7. Hand structured findings to the coding agent to fix.
8. Retest failed scenarios first until all gates are green.
