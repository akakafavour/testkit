---
name: architect
description: >-
  Lead Day-0 system design and planning for any application. Use when the user says
  "plan my app", "design the architecture", "choose tech stack", "set up data model",
  or "create system design". Gathers requirements, selects technology stack, models
  schemas and API contracts, and defines the independent invariants that serve as
  TestKit's verification oracle.
license: Apache-2.0
allowed-tools: [Read, Grep, Bash]
---

# Architect Skill (Day-0 Foundation)

This skill guides the builder from raw human intent to a complete, production-ready system blueprint.

## Workflow

1. **Requirements & Scope:**
   - Define target users, core workflows, platform limits, and non-functional requirements.
   - Produce `brief.md` and `plan.md`.

2. **Stack Selection:**
   - Choose runtime, backend framework, database, and infrastructure based on project profile and scale.
   - Respect user constraints ("analyse the decision made, never argue without measurement").

3. **Data Model, Contracts & Algorithmic Design:**
   - Define database tables, relations, and index types (B-Tree, Hash, GIN).
   - Evaluate algorithmic complexity upfront: enforce $O(\log N)$ or $O(1)$ lookups on hot paths (Prefix Tries for search, Min-Heaps for ranking, Inverted Indexes).
   - Define API contracts and role authorization matrix (Admin, User, Guest, Anonymous).

4. **Declare Invariants & Dynamic Stress-Testing Envelopes:**
   - Declare independent invariants *before* code is written.
   - **Zero Rigid Defaults:** Do NOT force a single hardcoded memory limit. The agent has the freedom to pick and choose any envelope (e.g., 200MB, 512MB, 4GB) or configure a progressive ramp:
     - Start tight (e.g., 200MB RAM) to catch subtle memory leaks early.
     - Increase incrementally under simulated load (stress-testing boundary limits until a break occurs).
   - Record invariants and chosen envelopes in `.testkit/brain/architecture.md`.

5. **Authority & Negative Constraints:**
   - **Supreme Source of Truth:** `.testkit/brain/architecture.md` is the immutable authority. Any coding agent that introduces unapproved frameworks, schema drift, or conflicting contracts is flagged for an invariant breach.
   - **Do NOT Invent Mock Stubs:** Never create fictitious data schemas or invent mock responses when a real schema or API contract has been declared.

6. **Handover to Coding Agent:**
   - Package the plan, architecture, and invariants so the coding agent can implement features directly to specification.
