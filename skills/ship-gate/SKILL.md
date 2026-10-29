---
name: ship-gate
description: >-
  Block commits, pull requests, or production deployment if unverified invariants
  or open confirmed Critical/High findings exist.
license: Apache-2.0
allowed-tools: [Bash, Read]
---

# Ship-Gate Skill

Enforces consequence. Operating rules:
- **Advice + a measurement = a gate (Blocks merge/ship)**
- **Advice without a measurement = advisory (Does not block)**

## Evidence Requirements & Proof Rules
1. **No Claims Without Receipts:** The conversational claim of an agent ("I tested it and it works") has zero authority. Releases are authorized ONLY when backed by a cryptographically signed receipt (`.testkit/receipts/run_XXXX.json`) whose SHA-256 signature validates.
2. **Hard Block on Open Breaches:** If ANY open `P0` or `HIGH` invariant violation exists in active receipts, this skill commands `testkit gate` to exit with code `1` and halt the PR or deployment.
3. **Receipt Release:** Signs and stamps the final verified release certificate only when 100% of declared invariants hold.
