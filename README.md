# TestKit

**Full Lifecycle AI Software Intelligence.** Mechanical grounding, independent invariants, hardware sympathy, and tamper-evident cryptographic receipts for software written by agents.

```
       [ Human Intent / Product Idea ]
                      │
                      ▼
            [ testkit init (< 1s) ] ───► Zero-prompt mechanical setup (.testkit/ + TESTKIT.md)
                      │
                      ▼
            [ Architect Agent ] ───────► Declares hard invariants & hardware budgets in architecture.md
                      │
                      ▼
            [ Coding Agent ] ──────────► Implements application features
                      │
                      ▼
            [ testkit verify ] ────────► Runs 2-8ms scenarios, probes CPU/RAM, checks invariants
                      │
                      ▼
            [ Signed Receipts ] ───────► .testkit/receipts/run_XXXX.json (SHA-256 tamper-evident)
                      │
                      ▼
            [ testkit gate ] ──────────► Hard pre-commit/CI blocker: Zero open P0 bugs, zero slop
```

TestKit is the agent's testing toolkit and operating manual. **The agent is the operator**, not an autonomous QA system: it does not invent requirements out of thin air, chase hallucinated bugs, or edit your source to fake passing tests.

---

## The Mental Model: The Remotion Analogy

Remotion does not make videos for you. It hands you a programmable toolkit and primitives (`Composition`, `Sequence`, `useCurrentFrame()`) that turn a vague task into a mechanical one. You still direct the video. TestKit does the exact same for software verification:

| Remotion Primitives | TestKit Primitives | Role in Verification |
| --- | --- | --- |
| `Composition`, `Sequence` | Scenario files in `.testkit/scenarios/*.json` | Declarative, reproducible test definitions |
| `useCurrentFrame()` | System clock virtualization (`time_jump: 14d`) | Fast-forward trial expiry, crons, billing in 2ms |
| The Render Pipeline | `testkit verify` → `.testkit/receipts/` | In-process scenario matrix & cryptographic receipts |
| You direct the video | The Agent authors scenarios and drives the UI | The Mind directs; the Engine provides the substrate |

Three coordinated surfaces:
1. **The Native Rust CLI (`testkit`)**: Blazing-fast (<5ms), on-demand, zero-daemon execution for scaffolding, invariant verification, telemetry sampling, and pre-commit gating.
2. **The 5 Agent Skills (`skills/`)**: The operating manual teaching any agent (`architect`, `verify`, `triage`, `replay`, `ship-gate`) how to think like a Principal Systems Engineer.
3. **The 4 Agent Personas (`agents/`)**: Specialized roles (`testkit-architect`, `testkit-tester`, `testkit-security`, `testkit-orchestrator`) that eliminate prompt bloat through strict separation of duties.

---

## Why Existing AI Code Fails (and How TestKit Fixes It)

1. **The Self-Grading Fallacy:** When the same AI writes both the code and the test, it tests what it *built*, completely missing its own blind spots. TestKit anchors all expectations **externally** in `architecture.md` invariants authored *before* code exists.
2. **The Cloud-Lazy Trap:** When an app gets slow, naive AI tells you to upgrade to a $200/mo AWS server. TestKit enforces **Hardware Sympathy**—it catches $O(N)$ full-table scans, memory leaks, and unbatched disk syncs right on your local machine.
3. **Context Overload & Skill Dilution:** Dumping 10 skills into one prompt causes hallucinations and instruction conflicts. TestKit enforces isolated agent-to-skill binding where each specialist loads only its assigned task.
4. **Source Tampering:** Standard agents often modify test assertions to force a failing test to turn green. TestKit enforces an iron rule: **TestKit never edits an invariant to make a test pass.** The agent must fix the application source code.

---

## Deep Engineering & Algorithmic Verification

TestKit does not merely test syntax. It enforces algorithmic efficiency and computer science fundamentals:

- **B-Trees & LSM-Trees:** Flags $O(N)$ sequential table scans on disk; proves $O(\log N)$ indexed query plans.
- **Prefix Tries & Inverted Indexes:** Enforces optimal prefix trees for search and autocomplete instead of linear array scans.
- **Topological Sorting:** Verifies async worker dependency DAGs to eliminate deadlocks and race conditions.
- **Hardware Latency Hierarchy:** Asserts the physical realities of RAM access (~100ns) vs. SSD I/O (~100µs) vs. Network (~150ms).
- **Adaptive Resource Ramps:** Zero rigid defaults. The agent can dynamically ramp testing budgets (e.g. 200MB → 500MB → 1GB RAM) to discover exact breaking points and memory creep under load.

---

## Repository Architecture

```text
testkit/
├── Cargo.toml                      # Clean Rust workspace config
├── package.json                    # OpenAgent / npm plugin distribution manifest
├── plugin.json                     # Agent Plugins 1.0 standard manifest
├── hooks/
│   └── hooks.json                  # Lifecycle interception hooks (pre-commit gate, edit sync)
├── agents/                         # Persona-driven system definitions
│   ├── testkit-architect.md        # System design & hard invariant author
│   ├── testkit-tester.md           # Scenario executor & evidence gatherer
│   ├── testkit-security.md         # Red-teaming & auth boundary auditor
│   └── testkit-orchestrator.md     # Phase coordinator & gatekeeper
├── skills/                         # AgentSkills 1.0 specifications
│   ├── architect/SKILL.md          # Invariant definition & .testkit/brain/ rules
│   ├── verify/SKILL.md             # Invariant execution & progressive ramps
│   ├── triage/SKILL.md             # Root-cause clustering & agent dispatch
│   ├── replay/SKILL.md             # 3/3 deterministic reproduction & delta checks
│   └── ship-gate/SKILL.md          # Receipt validation & PR/Deploy blocker
├── templates/
│   └── TESTKIT.md                  # Standard manual dropped into repos on `testkit init`
└── crates/
    ├── testkit-cli/                # Native binary CLI (init, verify, gate, receipt)
    ├── testkit-core/               # Engine core (telemetry, time-travel, receipts, evidence)
    └── testkit-daemon/             # Optional ambient background observer
```

---

## Installation & Setup

Requirements: Rust **1.77+**, `git`, Node.js **18+** (for plugin integration).

### Build from Source
```sh
cargo build --release
```

Outputs native binaries in `target/release/testkit` (`.exe` on Windows).

Install directly to your `PATH`:
```sh
cargo install --path crates/testkit-cli
# Or link via npm for agent plugins:
npm run install:local
```

---

## CLI Reference

TestKit runs as an on-demand, standalone command-line suite (zero background daemon required):

### 1. `testkit init`
Provisions the workspace in <1s with zero interactive prompts (identical to `git init`).
```sh
testkit init
# Or initialize a specific directory:
testkit init --path ./my-project
```
Creates `.testkit/` (`brain/`, `evidence/`, `receipts/`, `scenarios/`), configures `.gitignore`, and drops `TESTKIT.md`.

### 2. `testkit verify`
Executes scenarios against declared invariants and profiles host hardware usage (CPU/RAM).
```sh
# Run the entire scenario suite
testkit verify

# Run only scenarios affected by recent code changes
testkit verify --changed

# Filter by scenario ID or invariant tag
testkit verify --scenario trial-expiry-blocks-api
testkit verify --tag INV-012

# Override the hardware stress-test envelope
testkit verify --max-rss-mb 256 --max-cpu 80
```

### 3. `testkit receipt`
Inspects, parses, and validates cryptographically signed receipts.
```sh
# View summary of the latest test run
testkit receipt latest

# Inspect a specific receipt with full timeline and failure details
testkit receipt show rcpt_01J9Z4K2Q7

# Verify cryptographic SHA-256 signature and integrity
testkit receipt verify rcpt_01J9Z4K2Q7
```

### 4. `testkit gate`
Evaluates receipts to enforce quality gates in pre-commit hooks and CI/CD pipelines.
```sh
# Standard gate: exits with 1 if any open P0/HIGH failures exist, 0 if clean
testkit gate

# Quiet mode for shell hooks (pre-commit, CI)
testkit gate --quiet

# Advisory/warning mode (prints open issues without blocking exit code)
testkit gate --warn-only
```

---

## Cryptographic Receipts

A receipt is an immutable, canonical JSON document with a SHA-256 evidence hash and a cryptographic signature. Editing a result or timestamp invalidates it immediately:

```json
{
  "schema": "testkit.receipt/1",
  "id": "rcpt_01J9Z4K2Q7",
  "scenario_id": "trial-expiry-blocks-api",
  "status": "PASSED",
  "severity": "P0",
  "invariants_checked": ["INV-012"],
  "hardware_metrics": {
    "peak_cpu_percent": 37.5,
    "peak_rss_mb": 214,
    "wall_time_ms": 812,
    "envelope": { "max_cpu_percent": 95.0, "max_rss_mb": 1536 }
  },
  "violations": [],
  "evidence_hash": "sha256:8f1c...",
  "signature": "sha256:2ab7..."
}
```

- **`PASSED`**: All invariant expectations held.
- **`FAILED`**: An expectation was violated (crime scene captured).
- **`NOT_PROVEN`**: Claim could not be established (unparsable result, timeout).
- **`BLOCKED`**: Precondition failed (missing dependency, unreachable service, hardware envelope breached).

---

## The Two-Tier Brain

TestKit maintains a clean separation between local project truth and cross-project developer wisdom:

| Tier | Location | Scope & Purpose | Git Status |
| --- | --- | --- | --- |
| **Project Brain** | `.testkit/brain/architecture.md` | Invariants, API contracts, and domain rules for THIS repo. | **Tracked** |
| **Project Learned**| `.testkit/brain/learned-rules.json` | Project-specific patterns distilled from past receipts. | **Tracked** |
| **Project Evidence**| `.testkit/evidence/` | Raw screenshots, network traces, CPU/RAM profiles. | **Ignored** (`.gitignore`) |
| **Global Brain** | `~/.testkit/brain/global-knowledge.json` | Cross-project engineering wisdom (anti-patterns, memory leak lessons). | **Local to Machine** |

---

## OpenAgent / Plugin Integration

TestKit natively supports the **Agent Plugins 1.0** and **Agent Skills 1.0** specifications, seamlessly integrating with **Claude Code, OpenCode, Cursor, and Antigravity**.

`hooks/hooks.json` automatically enforces lifecycle discipline:
- **`PreToolUse` (Bash / `git commit`)**: Calls `testkit gate --quiet` to prevent commits if unresolved P0 invariant violations exist.
- **`PostToolUse` (Write/Edit)**: Calls `testkit --once-remap --quiet` to refresh AST state and mark affected scenarios stale without needing a background daemon.
- **`Stop`**: Warns if sessions conclude with active test failures.

---

## License
MIT. See `Cargo.toml` for workspace metadata.
