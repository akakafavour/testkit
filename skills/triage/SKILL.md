---
name: triage
description: >-
  Triage detected bugs, invariant failures, and security vulnerabilities into actionable,
  prioritized issues. Use when `testkit verify` uncovers findings, or when categorizing
  evidence receipts before routing to coding agents for fixes.
license: Apache-2.0
allowed-tools: [Bash, Read, Grep]
---

# Triage Skill

Systematically triages raw failure evidence and invariant breaches into prioritized fixes.

## Responsibilities
1. **Evidence Ingestion:** Read failure records and traces from `.testkit/receipts/` and `.testkit/evidence/`.
2. **De-duplication & Root Cause Clustering:** Group symptoms (e.g. 10 failing API calls) back to the single root cause (e.g. missing database migration or unhandled promise rejection).
3. **Severity Classification:**
   - **P0 (Blocker):** Data corruption, authentication bypass, IDOR, unhandled crash, memory leak exceeding host envelope. Blocks commit/deploy gates immediately.
   - **P1 (High):** Major functional regression, missing error boundaries, slow queries exceeding SLA budgets.
   - **P2 (Advisory):** Architectural bloat, suboptimal indexing suggestions, missing docstrings.
4. **Agent Dispatch:** Formulate concrete, reproduction-ready fix briefs with code locations and hand them to the relevant coding subagent (frontend, backend, database).
