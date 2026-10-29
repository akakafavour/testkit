---
name: replay
description: >-
  Deterministically reproduce and replay a bug or invariant violation from captured evidence.
  Use when reproducing a bug reported in `.testkit/receipts/`, confirming a fix before and after,
  or isolating intermittent race conditions without consuming LLM tokens.
license: Apache-2.0
allowed-tools: [Bash, Read, Grep]
---

# Replay Skill

Executes deterministic replay harnesses from captured TestKit receipts and evidence.

## Responsibilities
1. **Crime Scene Recreation:** Load the exact pre-seeded state, mock network inputs, and simulated timestamps recorded in the failure receipt.
2. **Deterministic Execution:** Execute compiled replay scripts without LLM token cost to confirm 3/3 failure reproduction.
3. **Differential Verification (Before vs. After):**
   - Run the replay script against the pre-fix commit (verify it fails).
   - Run the replay script against the post-fix commit (verify it passes).
   - Capture side-by-side screenshots and state snapshots (`before_fix` vs `after_fix`).
4. **Flake Analysis:** Distinguish intermittent timing races from deterministic logic bugs by running controlled iterations with simulated event-loop delays.
5. **Receipt Update:** Stamp the verified resolution into `.testkit/receipts/` with updated cryptographic proof.
