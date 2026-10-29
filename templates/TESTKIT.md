# This repository is TestKit-governed

TestKit is the agent's testing toolkit and operating manual. You are the operator. Do not treat a build, a manual click-through, or a green unit suite as proof that this application works.

## Where things live

| Path | Purpose |
| --- | --- |
| `.testkit/brain/architecture.md` | The contract and the numbered HARD INVARIANTS. This is the oracle. |
| `.testkit/brain/learned-rules.json` | Durable lessons from confirmed failures. Append only. |
| `.testkit/scenarios/*.json` | Executable scenarios, one invariant per file. |
| `.testkit/evidence/` | Screenshots, traces, logs, telemetry. Never commit this directory. |
| `.testkit/receipts/run_*.json` | Append-only, SHA-256 signed proof of every run. |
| `~/.testkit/brain/global-knowledge.json` | Cross-project knowledge. Read it; do not hand-edit it. |

## Non-negotiables

1. Do not mark a task done, open a release PR, or commit a feature until `testkit gate` passes.
2. Every P0/HIGH invariant needs a current-build `PASSED` receipt whose signature verifies. No receipt means not proven.
3. Never edit, delete, move, re-sign, or regenerate a receipt. History is evidence.
4. Never weaken an invariant, lower a severity, or change an expected value to make a run pass.
5. Never bypass the gate. Only a human release owner may override, in writing, and the override never turns a failure into a pass.
6. Keep `.testkit/evidence/` out of git and out of shared chat; it may contain sensitive state.

## The commands

```bash
testkit init --path .
testkit verify --path . --scenario <id>
testkit verify --path . --fail-fast
testkit gate --path .
testkit receipt list --path .
testkit receipt verify .testkit/receipts/run_<id>.json
```

`testkitd --project . --port 4949 --interval-ms 1000` runs the local daemon when a scenario needs live telemetry. Stop it when you are done.

## When the gate blocks

1. Read the blocking receipt IDs from `testkit gate --json`.
2. Verify the receipt is authentic: `testkit receipt verify <file>`.
3. Reproduce the breach: `testkit verify --scenario <id>`.
4. Run the failing scenario three times. Only 3/3 is a confirmed defect; record every run ID.
5. Fix the underlying breach in application code, migrations, or configuration.
6. Re-run the scenario and confirm a new signed `PASSED` receipt.
7. Re-run `testkit gate` on the exact commit you intend to ship.

Do not proceed to step 5 by editing the scenario to match current behavior, and do not proceed to step 7 with an open `FAILED` or `BLOCKED` receipt at P0/HIGH.

## Adding coverage

Add or extend a scenario in `.testkit/scenarios/` whenever you add a user-visible behavior, a money movement, a permission boundary, or a schedule. It must name an existing invariant; if none fits, ask the Architect to add one and sign the revision first. Use `time_jump` (`1h`, `7d`, `14d`, `30d`) for expiry and drift cases; TestKit injects `TESTKIT_FAKE_NOW` automatically.

Resource budget for test runs is the project ResourceEnvelope: 95% CPU and 1536 MB RSS on a 1 vCPU / 1.5 GB host. A scenario that exceeds it is a finding, not a reason to raise the limit.
