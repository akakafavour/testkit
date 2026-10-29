# TestKit — Expanded Operating Model Research

**Research date:** 2026-10-11  
**Purpose:** Validate the correction from “isolated copy plus a narrow web-testing loop” to a persistent, cross-platform, proactive software intelligence and verification layer.

## Findings

### 1. Persistent map plus flexible execution is the better abstraction

The product should separate the **project intelligence plane** from the **execution plane**. The intelligence plane persists the semantic map of source, runtime surfaces, requirements, roles, contracts, and evidence. The execution plane chooses among in-place local execution, a worktree, containers, emulators, simulators, VMs, cloud previews, or pre-production infrastructure.

This resolves the earlier overstatement that TestKit always creates an isolated copy. Isolation is important for destructive or untrusted work, but it is not the product definition.

### 2. UI and API should be combined rather than treated as competing layers

Playwright’s API-testing documentation explicitly supports direct API calls for testing server APIs, preparing state before a UI visit, and validating server-side postconditions after browser actions. This supports a TestKit model where the same goal can use API, UI, database, and runtime evidence together.

Source: https://playwright.dev/docs/api-testing

### 3. Platform coverage can be adapter-based

Maestro presents mobile and web UI testing through a CLI, human-readable flows, CI integrations, and agent support. Appium documents a broader driver ecosystem covering mobile, browsers, desktop, TV, and other platforms. TestKit should not claim that every target has identical capabilities, but it can use a common goal/evidence protocol with platform-specific adapters.

Sources:

- https://maestro.dev/
- https://appium.io/docs/en/2.0/

### 4. Observation must include more than application logs

OpenTelemetry semantic conventions cover traces, metrics, logs, profiles, and resources. TestKit should also support domain-specific events and product signals such as session replay, audit trails, feature flags, queues, background jobs, provider events, and customer feedback.

Source: https://opentelemetry.io/docs/concepts/semantic-conventions/

### 5. Session replay is structured interaction evidence

Sentry documents browser replay as lightweight DOM event logs, including clicks, scrolls, and mutations. A production witness can use this kind of evidence to reconstruct what a user experienced without assuming the system stores a literal video recording.

Source: https://docs.sentry.io/platforms/javascript/session-replay/understanding-sessions/

### 6. Ticket systems can become part of the evidence loop

GitHub’s Issues API supports state, labels, assignees, milestones, issue types, relationships, and suggestions. Linear exposes GraphQL mutations and webhooks for issue and comment changes. This enables a bidirectional workflow: an issue creates a candidate verification goal, and a confirmed finding updates the issue with evidence and replay status.

Sources:

- https://docs.github.com/en/rest/issues/issues
- https://linear.app/docs/api-and-webhooks

### 7. Always-on autonomy needs policy and visible approvals

Claude Code’s permission modes show a useful pattern: manual approval, accepted edits, automated review, pre-approved tools, and bypass modes reserved for isolated environments. TestKit should expose a host-neutral policy with allow/ask/deny/budgeted actions and translate it into hooks and permissions for each coding-agent host.

Source: https://code.claude.com/docs/en/permission-modes

### 8. Stable replay is a product trust problem, not only a test-runner feature

AI-generated tests can vary, and an AI evaluator can call a bug a bug when the software is behaving correctly. TestKit should stabilize the result through versioned goals, deterministic compiled scripts, fixed seeds, controlled dependencies, independent expectations, negative controls, replay thresholds, flake history, and four-way verdicts. Probabilistic AI output should be judged with explicit tolerances or developer-owned evaluators.

The durable artifact is not an AI opinion. It is a structured, replayable evidence record whose verdict remains consistent for the same goal, commit, environment, and policy.

### 9. Some failures are operationally emergent

Production-only or production-dependent failures can arise from traffic distributions, long-lived data, rare scheduling, provider changes, device/network combinations, rollout behavior, feature cohorts, and human operational actions. TestKit should therefore implement shift-left plus shift-right: pre-release verification and post-release read-only observation with safe reproduction and regression harvesting.

## Product implication

TestKit should be positioned as:

> **A persistent, permission-controlled software collaborator that maps a codebase, understands goals, uses the right interface for each target, observes the entire system, produces replayable proof, learns from real failures, guides architecture and release decisions, and helps the coding agent fix and re-verify the result.**

It is broader than a test runner, but its trust boundary remains strict: it should not silently make source changes, perform destructive production actions, or turn uncertain observations into confirmed bugs.

## Additional findings from IDE and workflow research

### Agentic hosts already expose the integration surface TestKit needs

Android Studio Agent Mode handles multi-stage tasks, invokes tools, builds, inspects Logcat, deploys to devices, inspects screens, and waits for approval. Xcode provides an MCP bridge that lets external agents use project and build capabilities after the user enables access. Flutter’s official AI plugin model bundles skills, persistent rules, a Dart/Flutter MCP server with analyzer and test state, and specialized agents. TestKit should therefore be a portable agentic plugin that attaches to host tools instead of a separate application that replaces them.

Sources:

- https://developer.android.com/studio/gemini/agent-mode
- https://developer.apple.com/documentation/xcode/giving-external-agents-access-to-xcode
- https://docs.flutter.dev/ai/get-started

### Immediate remapping must be incremental

The semantic map should update as soon as a relevant save, rename, dependency/config/schema edit, IDE event, Git event, or runtime observation occurs. A full repository rescan for every keystroke is wasteful. File watchers, language servers, analyzers, compiler diagnostics, Git hooks, and runtime adapters should update affected nodes and mark uncertain nodes stale.

### Git hygiene is part of software assurance

GitHub documents that a repository `.gitignore` can keep generated files out of commits, but ignored rules do not affect files already tracked. TestKit should create or propose scoped ignore rules for caches, temporary environments, emulator state, local secrets, raw telemetry, logs, recordings, test databases, and unredacted evidence. It must preserve tracked project-owned manifests, approved goals, policies, and selected fixtures. GitHub push protection should be treated as a separate secret-control layer because it blocks detected credentials before they reach the repository.

Sources:

- https://docs.github.com/articles/ignoring-files
- https://docs.github.com/en/code-security/concepts/secret-security/push-protection

### Environment separation and code handover are measurable quality concerns

The Twelve-Factor configuration principle separates deploy-varying values and credentials from code, which gives TestKit a concrete check for hardcoded secrets and environment drift. Google’s TypeScript guidance emphasizes long-term maintainability, explicit dependencies, tests, and automated checks for rules that can be enforced mechanically. These support TestKit’s role as a handover-quality gate for AI-generated code, without reducing quality to line count.

Sources:

- https://12factor.net/config
- https://google.github.io/styleguide/tsguide.html

### Expanded production learning includes abuse and free-tier economics

Production learning should include not just technical failures but also abuse of the product: free-tier exploitation, quota exhaustion, scraping, spam, bot activity, fraudulent workflows, unusual retry patterns, and cost-amplifying usage. These signals should be privacy-preserving and policy-controlled, then converted into safe simulated goals or regression cases after confirmation.

## Core and plugin boundary

The corrected architecture has two layers. The **TestKit Core** is a local, deterministic, Git-like tracker: it watches files and project events, updates the semantic map, maintains indexes and evidence, and exposes structured state. It does not need an LLM for routine work and should not burn tokens merely to stay current.

The **TestKit Agent Plugin** is the optional reasoning and workflow package. Claude Code’s official documentation defines a plugin as a directory that can bundle skills, agents, hooks, MCP servers, and other components, with a manifest at `.claude-plugin/plugin.json` that can declare component paths. Claude skills load on demand, and side-effectful skills can be made user-invocable rather than model-invocable. This maps directly to TestKit: the plugin can provide `/testkit:map`, `/testkit:test`, `/testkit:debug`, `/testkit:review`, `/testkit:handover`, and `/testkit:release-check`, while the core keeps tracking locally between agent invocations.

Sources:

- https://code.claude.com/docs/en/plugins/overview
- https://code.claude.com/docs/en/plugins/manifest-reference
- https://code.claude.com/docs/en/skills

This separation also improves portability. Other hosts can use the same core through its CLI, local API, MCP surface, hooks, or native adapter, while Claude Code receives the richest initial package of skills, subagents, hooks, commands, and MCP tools.
