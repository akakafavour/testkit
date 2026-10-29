# TestKit — Full Product Explanation and Expanded Operating Model

**Reviewed:** 2026-10-11  
**Scope:** TestKit material and research in this workspace, excluding the AgentAuth files and `03-colleague-prd.md`.  
**Current authority:** `16-testkit-product-scope-v2.md` for scope, `09-testkit-plugin-architecture.md` for V1 packaging, and `17-testkit-capability-map-by-mode.md` for operating modes.

## The corrected definition

TestKit is a **persistent software intelligence, verification, debugging, and delivery-control layer** that is initialized into a codebase in the same general way a project is initialized with Git.

It does not necessarily clone the entire application into a separate environment every time. After initialization, it can maintain a semantic map of the codebase and use the developer’s normal project, running services, terminals, simulators, emulators, and tools according to the project’s configuration and permission settings.

Isolation is a **mode**, not the definition of the product:

- Normal local development can be monitored and tested in place.
- A disposable copy, worktree, container, emulator, or VM can be used when a test is destructive, parallel, untrusted, expensive, or needs clean state.
- A cloud or pre-production environment can be used when the developer explicitly wants deployment rehearsal, scale, cross-machine testing, or production-like infrastructure.

The core product is the persistent map, deterministic change tracking, evidence system, replay system, and local policy engine—not the sandbox and not an autonomous agent. The core runs continuously without an LLM and therefore does not burn tokens while it watches the repository.

> **TestKit continuously understands what the software is, operates it through the interfaces real users and systems use, measures what actually happens, finds and explains deviations from requirements, keeps proof that can be replayed consistently, and helps the developer or coding agent decide and act safely.**

## Two-layer architecture

TestKit has two deliberately separate layers:

**The TestKit Core** is a local TypeScript/Node.js daemon and CLI, conceptually closer to Git than to an AI agent. It watches files and project events, updates the semantic map, records provenance, maintains indexes, detects affected areas, runs deterministic checks and hooks, stores evidence, and exposes structured state. It uses no model for routine tracking. A developer can run the core with no Claude, no network, and no token budget.

**The TestKit Agent Plugin** is the optional portable package that lets Claude Code or another compatible host use the core intelligently. It contains skills, subagents, hooks, commands, MCP configuration, rules, adapters, and references. An agent is invoked when a human asks for planning, testing, debugging, architecture advice, ticket handling, code review, or a fix/retest loop. The plugin reads the core’s current structured state instead of repeatedly rediscovering the repository in a prompt.

Conceptually:

```text
TestKit Core (always on, deterministic, no LLM)
  ├─ file/Git/IDE/runtime watchers
  ├─ semantic map and change graph
  ├─ evidence, replay, indexes, and local policy
  └─ CLI + local API/MCP surface

TestKit Agent Plugin (invoked when useful)
  ├─ skills / reusable workflows
  ├─ subagents / delegated specialist roles
  ├─ hooks / lifecycle automation
  ├─ commands / explicit user entry points
  ├─ MCP server / structured tools
  └─ host and platform adapters
```

## How TestKit is initialized

A developer runs something conceptually similar to:

```bash
npx testkit init
```

Initialization does not mean “copy the whole app and throw the original away.” It registers TestKit with the project and detected agent hosts, creates project configuration, and starts building a **living semantic map**.

The map can include:

- Repository structure, files, packages, modules, symbols, and imports.
- Frameworks, languages, runtimes, versions, and configuration.
- Routes, screens, UI controls, accessibility labels, and navigation paths.
- API endpoints, events, webhooks, queues, jobs, and contracts.
- Database schemas, migrations, indexes, constraints, policies, and relationships.
- Authentication flows, roles, permissions, tenants, and authorization boundaries.
- State machines: loading, empty, success, partial success, failure, timeout, offline, retry, and recovery states.
- Infrastructure, deployment files, containers, environment variables, cloud resources, and CI workflows.
- Existing tests, fixtures, mocks, observability integrations, issue trackers, and runbooks.
- Native application screens, platform capabilities, deep links, notifications, background work, and device permissions.
- CLI commands, desktop windows, services, libraries, embedded systems, and backend-only components where adapters exist.

The map is not a one-time generated document. It is an always-on index that reacts immediately to code changes. A save, rename, dependency edit, schema change, configuration change, generated-file change, commit, branch switch, running-app observation, new issue, production signal, or accepted requirement invalidates and refreshes the affected map nodes before the next agent action. The map records confidence and provenance so TestKit can distinguish “found in source,” “observed at runtime,” “declared by the developer,” and “inferred.”

The map update path should be incremental rather than a full rescan after every keystroke. File watchers, language servers, compiler/analyzer diagnostics, Git hooks, IDE events, package-manager events, runtime inspection, and adapter-specific watchers can update only the impacted symbols and relationships. If an update is incomplete, TestKit marks the affected area stale and tells the agent instead of pretending the map is current.

## TestKit is delivered as a portable agentic plugin, not a separate destination

TestKit’s agent layer is packaged like a real Claude Code plugin, while the deterministic core remains independently usable. The plugin directory can contain a manifest, skills, subagent definitions, hooks, commands, an MCP server configuration, LSP or platform adapters, and reference material. Claude Code’s official plugin model supports a directory containing skills, agents, hooks, MCP servers, and other components loaded as one unit; its manifest supports explicit paths for those components.[16][17]

The plugin should expose namespaced commands such as `/testkit:map`, `/testkit:test`, `/testkit:debug`, `/testkit:review`, `/testkit:handover`, and `/testkit:release-check`. Skills should load only when relevant or explicitly invoked. Side-effectful workflows such as deployment, ticket creation, branch changes, or destructive testing should be explicitly user-invocable and permission-gated. Claude Code’s skills model supports this separation between model-invocable knowledge and user-invocable actions.[18]

The plugin can attach to terminal agents, editor agents, IDE agents, and platform-native agents. The developer should be able to use it inside the workflow they already use rather than moving the project into a separate TestKit application.

This is practical today: Android Studio Agent Mode can plan multi-stage work, invoke tools, build, inspect Logcat, deploy to devices, inspect screens, and wait for approval; Xcode can expose project and build capabilities to external agents through its MCP bridge; and Flutter’s official agent plugins combine skills, persistent rules, MCP access to analyzer and test state, and specialized agents.[9][10][11] TestKit should integrate with those capabilities rather than compete with them.

## What the map is used for

The map is the shared context for every TestKit capability:

1. Select the areas affected by a change.
2. Find workflows that cross the changed code.
3. Generate scenarios a human may not have written.
4. Identify missing states and untested boundaries.
5. Decide which tools and environments are needed.
6. Observe the right signals while a goal is being performed.
7. Explain how a finding relates to code, configuration, or an external dependency.
8. Keep the next run focused instead of starting from zero.

TestKit can work directly from a normal working tree during safe read-only analysis. It can use a worktree, branch, container, emulator snapshot, or VM when it needs separation. It can run against local development, a preview, a staging environment, or an explicitly selected cloud pre-production target. Production access is a separate policy-controlled witness mode and is not the same as destructive testing.

## TestKit is goal-oriented, not only test-case-oriented

A user may ask:

- “Test the new checkout goal.”
- “Why are messages duplicated?”
- “Can an ordinary user reach any admin capability?”
- “Will this architecture scale to 10,000 tenants?”
- “Prepare this app for deployment.”
- “Watch this project while I build it.”
- “Investigate this customer bug report.”
- “Review this pull request.”
- “Keep the project within the agreed reliability and security rules.”

TestKit turns the request into a goal, identifies the relevant parts of the map, plans actions and observations, and returns a structured result. A goal is different from a fixed test script: the same goal can be checked through API calls, real UI actions, native controls, database state, messages, files, device sensors, or command-line behavior depending on the software.

## The evidence loop

```text
initialize and map
  → understand the goal and its independent expectations
  → choose the least risky valid execution mode
  → select tools, actors, environments, and observers
  → operate the software through the relevant surfaces
  → collect broad evidence and measurements
  → compare behavior with requirements, invariants, contracts, and policies
  → find a causal chain and classify confidence
  → replay the goal and verify the result is stable
  → report, ticket, or ask for permission to act
  → coding agent or authorized TestKit action changes something
  → remap affected areas and re-check regressions
  → preserve the case as a durable learning and regression asset
```

## The execution modes

### In-place development mode

TestKit watches and reasons about the project while the developer works. It can remap changed files, inspect running processes, monitor test results, notice a changed dependency, identify stale scenarios, and prepare a focused verification plan. It should be low-resource and visible. Heavy actions require a configured permission level.

### Local active testing

TestKit uses the developer’s normal app and tools where safe. It may drive a browser, call an API, run a CLI, interact with a desktop window, use a mobile emulator, exercise a simulator, send a message, or inspect an output file. It can prepare state through APIs and then verify the corresponding real UI. Playwright explicitly supports this hybrid approach: direct API calls can prepare state and validate server-side postconditions around browser actions.[1]

### Disposable isolated testing

A clean environment is used when the test needs destructive actions, service failure, parallel execution, untrusted code, database resets, dependency substitution, or reproducible state. The environment may be a worktree, container, emulator snapshot, simulator, local VM, or remote microVM.

### Cloud and pre-production mode

A cloud or pre-production environment is useful for scale, real deployment topology, managed services, device farms, geographic behavior, provider integrations, and release rehearsal. TestKit should understand that this environment is not automatically safe. It must know whether it is a disposable preview, shared staging, or live production and enforce the appropriate policy.

### Production witness mode

Production can be observed in read-only mode when explicitly enabled. TestKit can learn from real incidents, errors, traces, user journeys, tickets, support reports, performance patterns, feature cohorts, fraud signals, abuse, free-tier exploitation, quota exhaustion, spam, scraping, bot behavior, and operational mistakes, then reproduce or simulate them elsewhere. It does not send synthetic destructive traffic to live users by default. Production evidence is sanitized, permission-scoped, retained according to policy, and converted into regression goals only after confirmation.

## What TestKit watches

“Frontend, API, server, database, queues, and logs” is too narrow. TestKit should use an **adapter-based observation fabric** and collect whatever signals are relevant and permitted.

### Code and static structure

- Source and generated code.
- AST and symbol relationships.
- Types, lint, complexity, duplication, dead code, and dependency graphs.
- Build output and compiler diagnostics.
- Configuration, secrets handling, feature flags, and environment drift.
- Infrastructure-as-code and deployment manifests.
- API schemas, event schemas, migration plans, and compatibility contracts.

### User and interface behavior

- Browser DOM and accessibility trees.
- Clicks, taps, keyboard input, gestures, focus, scrolling, dialogs, permissions, and navigation.
- Screenshots, layout breakage, visual state transitions, and interaction timing.
- Mobile and desktop control trees.
- Deep links, notifications, background/foreground transitions, app restarts, orientation, connectivity, and device capabilities.
- CLI prompts, exit codes, terminal output, file output, and interactive sessions.

Playwright covers web and API behavior. Maestro supports mobile and web flows with CLI and CI integration.[2] Appium provides a broad automation ecosystem for mobile, browser, desktop, TV, and other platforms through drivers and plugins.[3]

### Runtime and distributed-system signals

- HTTP, GraphQL, RPC, WebSocket, SSE, and message-bus traffic.
- Traces, spans, baggage, correlation IDs, and causal links.
- Logs, structured events, audit records, and error reports.
- Metrics, histograms, percentiles, saturation, throughput, and error budgets.
- Profiles, CPU, memory, allocations, garbage collection, event-loop delay, and thread behavior.
- Disk, network, file descriptors, sockets, process health, and container health.
- Database queries, locks, transactions, replication, migrations, indexes, row counts, checksums, and invariant violations.
- Queues, retries, dead letters, scheduled jobs, cron execution, and worker heartbeats.
- Cache hit rates, invalidation events, circuit breakers, rate limits, and backpressure.
- Object storage, email delivery, push notifications, payment events, webhooks, and third-party API responses.

OpenTelemetry supplies a useful common vocabulary across traces, metrics, logs, profiles, and resources, with semantic conventions that give common names to operations and data.[4] TestKit should adopt compatible naming while retaining project-specific evidence.

### Product and user feedback signals

- Session replay and interaction events.
- Error reports and crash reports.
- Support conversations and customer complaints.
- GitHub issues, pull requests, discussions, and comments.
- Linear issues, comments, projects, cycles, and webhooks.
- Incident-management alerts and postmortems.
- Survey results, app-store reviews, analytics funnels, and feature-flag exposures.
- Runbooks, requirements, design decisions, and acceptance criteria.

Session replay is not necessarily video. Sentry documents browser replay as lightweight DOM event logs such as clicks, scrolls, and mutations that can be retained in a buffer and uploaded around an error.[5] TestKit should consume these events as evidence while respecting privacy, consent, redaction, retention, and regional rules.

### TestKit uses specialist tools as evidence adapters

These tool families are part of the **full-product plan**, not a claim that TestKit must rebuild each
specialist tool. TestKit should invoke or ingest supported tools behind replaceable, permission-aware
adapters and join their results to the same evidence graph:

- **Browser and runtime:** Playwright/Stagehand for controlled actors; BrowserTools MCP for explicitly
  approved evidence from the developer's existing logged-in browser; Chrome DevTools MCP/CDP for
  supported browser inspection, console/network capture, and performance traces.
- **UI quality:** Lighthouse and axe-core for measurable page and accessibility audits; Percy/Chromatic-
  class tools for visual comparison against approved baselines. Visual differences need review, and
  automated accessibility checks disclose incomplete/manual coverage.
- **Security:** ZAP and compatible security MCPs for authorized, bounded DAST; TestKit's own actors,
  role/tenant matrix, and independent invariants for application-specific authorization and business
  logic. Active security testing runs in a sandbox/pre-production target, never production.
- **Production learning:** OpenTelemetry and Sentry/LogRocket/Datadog-class integrations for opt-in,
  read-only, minimized signals and session evidence. TestKit reproduces candidate failures on a branch
  or sandbox, not against live users.
- **Existing QA systems:** optional import/invocation adapters for existing runners or platforms such
  as Momentic and Suitest; their output retains original provenance and does not replace TestKit's
  independent expectation or receipt.

Every adapter contributes tool/version, target, actor, expectation provenance, artifacts, redaction
state, causal links, replay status, and coverage. An unavailable or unconfigured required capability
must be visible as blocked or unproven, never silently passed. The full adapter contract, safety
policy, evidence schema, and implementation workstreams are in
[`18-testkit-tool-adapter-and-evidence-integration-plan.md`](../18-testkit-tool-adapter-and-evidence-integration-plan.md).

## Actors and roles are application-specific

TestKit must not assume every application has “admin” and “standard user.” It should discover or ask about the roles that actually matter:

- Guest, registered user, verified user, suspended user, moderator, operator, manager, owner, auditor, support agent, billing administrator, tenant administrator, service account, API client, device, worker, or anonymous caller.
- Roles that change by tenant, project, resource, subscription, region, device trust, time, approval state, or feature flag.
- Conflicting or combined roles.
- Users with partially completed onboarding.
- Users with expired, revoked, or rotated credentials.
- Concurrent human and machine actors.
- External systems, webhooks, scheduled jobs, queues, and provider identities.

The role model is derived from the application’s own authorization matrix and requirements. TestKit should ask the user when the model is incomplete rather than inventing universal personas.

### Semantic actors run deterministically by default

A semantic actor is a scoped test identity plus a small deterministic service that executes allowed
application actions. It is **not** one autonomous LLM agent per test user. The actor uses an approved
API/domain adapter for breadth and state setup; a UI actor separately checks critical journeys through
the actual browser or device so API success is never mistaken for proof that the UI works.

An LLM can propose a scenario once, after which the reviewed scenario is compiled to a script and
replayed without model calls. If exploration reaches a genuinely unknown state, a bounded-live profile
may let a model choose from a small allow-list at that decision point. A deterministic policy checks
the choice before execution and enforces role, tenant, target, allowed actions, step/request/time caps,
and a token/call budget. The model cannot access arbitrary tools or expand its own scope. Unknown
states or exhausted budgets stop with an uncovered/blocked result; they do not trigger unlimited
improvisation. Every model choice is recorded, and confirmed bugs are reproduced by script without
asking the model to recreate them. The full actor contract is specified in
[`18-testkit-tool-adapter-and-evidence-integration-plan.md`](../18-testkit-tool-adapter-and-evidence-integration-plan.md).

This keeps ordinary regression runs token-free while retaining an optional, narrow way to use AI for
exploration when deterministic models of the app are incomplete.

## Scenario generation and stable replay

TestKit generates candidate goals from:

- User requirements and acceptance criteria.
- The semantic codebase map.
- API and event contracts.
- Authorization rules.
- Database and business invariants.
- Known bug-pattern taxonomy.
- Dependency and framework documentation.
- Existing tests and production history.
- Customer reports and issue tickets.
- Changed code and risk signals.

Generated scenarios are not automatically trusted. Each scenario records its source and expected behavior. The developer or authorized policy can review, pin, edit, disable, or approve it.

### The consistency problem

AI-generated tests can produce false positives and false negatives. TestKit should not rely on an AI model giving the same prose answer every time. It should make execution and verdicts as deterministic as possible:

- Compile approved goals into versioned scripts or plans.
- Use fixed seeds and controlled clocks where possible.
- Mock or record unstable external providers when appropriate.
- Compare structured states, invariants, schemas, and measurements rather than free-form model judgments.
- Require replay confirmation before calling a finding confirmed.
- Run a no-bug control and seeded known-bug benchmarks.
- Track flake history and environment differences.
- Separate `PROVEN`, `FAILED`, `NOT_PROVEN`, and `BLOCKED`.
- Label probabilistic AI outputs with tolerances and evaluation policies.
- Never create a bug merely because an AI agent expected a different answer.

For AI features, deterministic contract checks should usually cover schema, safety boundaries, tool calls, latency, cost, streaming, refusal behavior, and degradation. Content quality can use developer-supplied evaluators, reference datasets, or human review.

The objective is stable evidence: replaying the same goal against the same commit and controlled environment should produce the same structured verdict, or clearly explain why it could not.

## Testing, debugging, and fixing

TestKit should work with the coding-agent loop that developers already use:

1. TestKit detects or receives a goal.
2. It returns structured findings and supporting evidence.
3. The coding agent reads the finding and edits the code in the normal way.
4. Hooks or the agent invoke TestKit again.
5. TestKit remaps the changed area, replays the original goal, and runs impacted regressions.
6. The agent continues until the configured readiness policy is satisfied.

TestKit should not pretend that sending a report to an agent is the whole integration. It should provide the agent with machine-readable findings, reproduction commands, relevant files and symbols, causal evidence, expected behavior, and the exact verification status. The agent can then propose a fix, make the edit, run the verification command, and request approval when the next action exceeds policy.

## Proactive always-on operation

An always-on TestKit agent can maintain a project watch loop, but it must be governed and resource-aware.

It can proactively:

- Notice changed code and remap affected surfaces.
- Detect new routes, permissions, dependencies, migrations, and external integrations.
- Flag stale tests or scenarios.
- Run lightweight checks automatically.
- Schedule heavier tests after changes or during configured windows.
- Compare the current branch to the last green state.
- Detect a likely regression before the developer asks.
- Watch issue trackers, support signals, error streams, and production witnesses.
- Create a candidate bug report with evidence.
- Group duplicate findings by probable root cause.
- Propose a fix plan to the coding agent.
- Apply only the actions permitted by policy.
- Keep the user informed with a queue of planned, running, blocked, and completed work.

### Permission-controlled autonomy

TestKit needs more than “on or off.” Its policies should distinguish:

- Read-only analysis.
- Local edits by the host coding agent.
- Running safe local commands.
- Installing tools or dependencies.
- Starting services and emulators.
- Creating branches or worktrees.
- Creating or updating tickets.
- Opening pull requests.
- Sending external notifications.
- Accessing staging or cloud resources.
- Accessing production telemetry.
- Deploying or changing infrastructure.
- Destructive tests.
- Security tests with external effects.

Each action can be `allow`, `ask`, `deny`, or `allow within budget`. Claude Code’s current permission model demonstrates the useful distinction between manual approval, accepted edits, automated checks, pre-approved tools, and isolated bypass modes.[6] TestKit should expose a host-neutral policy and translate it into the permissions and hooks of each agent host.

The system must show the user what it plans to do, what it already did, what it wants to do next, and what is waiting for approval. It should stop rather than guess when an action changes data, spends money, contacts third parties, deploys, or widens access.

## Bugs that only appear in production

TestKit must be honest that some bugs cannot be reproduced before launch. They can depend on:

- Real traffic distributions.
- Rare timing and scheduling.
- Device, browser, carrier, region, or network combinations.
- Data volume and long-lived state.
- Third-party provider changes.
- Real user behavior and support workflows.
- Feature-flag cohorts.
- Clock, timezone, daylight-saving, and locale effects.
- Capacity limits and dependency brownouts.
- Partial failures across services.
- Operational procedures and human actions.
- Data migrations accumulated over months or years.

That does not make pre-production testing pointless. It means TestKit needs **shift-left and shift-right** operation:

- Before release: test the known contract, risk patterns, failure modes, scale model, and deployment procedure.
- After release: observe reality, detect anomalies and user-visible failures, minimize evidence, reproduce safely, and convert the case into a regression goal.

A production finding can be classified as reproduced, partially reproduced, environment-specific, not reproduced, or blocked. “We could not reproduce it” must not be converted into “there was no bug.”

## Tickets, bug reports, and developer collaboration

TestKit should connect findings to the team’s existing work systems rather than creating an isolated report graveyard.

A confirmed finding can become:

- A GitHub issue with reproduction, severity, evidence links, labels, affected commit, and replay command.
- A Linear issue with the same structured evidence and an update webhook.
- A pull-request comment or check annotation.
- A notification to the responsible team or on-call channel.
- A regression goal committed to the project.
- A postmortem input with timeline and contributing factors.

GitHub’s Issues API supports issue state, labels, assignees, milestones, issue types, relationships, and suggestions.[7] Linear provides API mutations and webhooks for issue and comment changes, which can trigger verification or update calculations.[8]

The ticket loop should be bidirectional:

```text
customer report / issue / alert
  → TestKit reads and classifies it
  → asks for missing information when needed
  → creates a candidate goal
  → reproduces in the safest available environment
  → attaches evidence and confidence
  → opens or updates the ticket
  → coding agent proposes a fix
  → TestKit verifies the fix
  → ticket is updated and regression goal retained
```

The product must avoid flooding teams with speculative tickets. Default behavior should be candidate finding first, confirmation second, ticket creation according to policy.

## Architecture and scaling guidance

TestKit should guide developers toward scalable decisions without forcing one universal stack. It can inspect:

- Data access and indexing.
- Connection pools and concurrency.
- Caching and invalidation.
- Queue and worker design.
- API pagination and idempotency.
- Rate limits and backpressure.
- Storage and retention.
- Multi-tenant isolation.
- Authentication and authorization.
- Observability coverage.
- Deployment topology.
- Recovery and rollback.
- Cost per request and dependency usage.

Advice becomes a gate when it has measurable evidence. “This endpoint has no idempotency key and duplicates rows under a replayed timeout” can block. “A monorepo may be easier for your team” is advisory.

For greenfield work, TestKit can help choose a reasonable architecture from requirements and constraints. For existing work, it should explain consequences first and propose alternatives when the user asks or when the current design violates an explicit requirement.

## CI/CD, deployment patterns, and release control

TestKit should understand and guide multiple deployment patterns:

- Single-server deployments.
- Containers and Docker Compose.
- Kubernetes rolling updates.
- Blue-green deployments.
- Canary releases.
- Feature-flag releases.
- Mobile app store releases.
- Desktop installers and auto-updates.
- Serverless and edge functions.
- Database migration releases.
- Queue and worker version transitions.
- Multi-region or active-active systems.
- Monorepo and multi-service pipelines.

It should explain what each pattern means for the application’s specific risks. For example, a migration may need expand-and-contract sequencing if old and new application versions run together. A canary needs metrics that reveal correctness and business impact, not only process health. A mobile rollout may need backward-compatible APIs because old clients remain active.

TestKit can generate or update GitHub Actions, GitLab CI, Jenkins, or other workflow configuration when authorized, but it must show the proposed changes and preserve the user’s control. It should understand the project’s existing workflow rather than blindly installing a new pipeline.

## Git hygiene and generated artifacts

`testkit init` should create or update project-local ignore rules so runtime state and generated material do not accidentally enter the repository or a deployment artifact. Generated caches, temporary environments, downloaded browsers, emulator state, local secrets, raw telemetry, large recordings, test databases, logs, and unredacted evidence belong under ignored paths such as `.testkit/cache/`, `.testkit/runtime/`, and `.testkit/evidence/`.

The rule is not “ignore everything TestKit creates.” Durable project-owned material must remain reviewable and versioned: the TestKit manifest, approved goals, policies, redaction rules, adapter configuration, migration checks, and selected regression fixtures. TestKit should propose the `.gitignore` change, detect already-tracked generated files, warn before untracking anything, and never hide a secret merely by ignoring it. GitHub’s documentation confirms that `.gitignore` controls what is ignored for commits but does not affect files already tracked; secret push protection is a separate preventative control.[12][13]

TestKit should also check that environment-specific configuration is separated from code. Database handles, provider credentials, API keys, hostnames, and deploy-specific values should be supplied through the project’s approved environment or secret manager, not hardcoded into source. The Twelve-Factor guidance describes this separation as a way to vary configuration between deploys without changing code.[14]

## AI code quality and handover readiness

TestKit should help an AI coding agent produce code that another developer can understand and maintain. “It runs” is necessary but not sufficient. The quality loop should check:

- The smallest clear change that satisfies the requirement.
- Existing abstractions reused instead of duplicated.
- No unnecessary files, wrappers, indirection, or speculative framework layers.
- Clear names, narrow responsibilities, and understandable control flow.
- Correct types, error handling, and boundaries.
- No dead code, debug output, accidental secrets, or unexplained magic values.
- Tests and fixtures that prove behavior without excessive brittle setup.
- Documentation for non-obvious decisions and operational behavior.
- Consistent formatting, lint, compiler settings, and project conventions.
- Complexity, duplication, dependency, bundle, query, and performance budgets where relevant.
- A clean handover summary explaining what changed, why, what remains uncertain, and how to operate it.

TestKit should not enforce a universal aesthetic or reward fewer lines blindly. A 50-line implementation is not automatically better than a 150-line implementation; the test is whether the code is proportionate, explicit, maintainable, and correct. It can flag duplicated logic, needless abstraction, excessive branching, generated spaghetti, and code that future maintainers cannot explain. Google’s TypeScript guidance similarly emphasizes long-term maintainability, automated checks for enforceable rules, explicit dependencies, and tests.[15]

The handover goal is that a professional developer can take over an AI-built codebase without first reverse-engineering hidden agent decisions. TestKit should maintain a project handbook containing the architecture map, conventions, invariants, operational commands, deployment assumptions, known limitations, and replayable verification goals.

## Non-web software

TestKit is not limited to web applications. The protocol should support thin adapters for:

- REST, GraphQL, RPC, and event-driven backends.
- Web applications.
- Android, iOS, React Native, Flutter, and other mobile applications.
- Windows and macOS desktop applications.
- CLI tools and terminal applications.
- Background workers and scheduled jobs.
- Browser extensions.
- APIs and SDKs.
- Data pipelines and batch jobs.
- Local services and distributed systems.
- Games and interactive clients where reliable instrumentation exists.

The same goal can span several targets. For example, a mobile action may call an API, create a database row, enqueue a job, send a notification, and update another user’s screen. TestKit should preserve that cross-surface evidence chain.

TypeScript/Node.js remains the V1 control-plane foundation because it fits the agent, CLI, and automation ecosystem. It does not limit the target software. TestKit can invoke platform-native drivers and tools, including Playwright, Maestro, Appium, device simulators, desktop accessibility APIs, command-line processes, and provider SDKs. Rust remains optional future optimization only where profiling justifies it.

## What TestKit is not

TestKit is not merely:

- A Playwright wrapper.
- A screenshot checker.
- A static analyzer.
- A load tester.
- A bug tracker.
- A deployment platform.
- A production monitoring dashboard.
- An AI that invents random failures.
- An AI that edits code without oversight.
- A guarantee that a system has no remaining bugs.

It is the **coordination, semantic understanding, evidence, replay, policy, and feedback layer** that connects these capabilities into one durable loop.

## Recommended product evolution

1. **Persistent map:** initialize into a project and maintain source/runtime semantics.
2. **Goal and evidence protocol:** define structured goals, expectations, evidence, verdicts, and replays.
3. **Local safe operation:** support in-place analysis plus optional isolated runs.
4. **Reference adapters:** web/API first, then mobile, desktop, CLI, workers, and distributed services.
5. **Deterministic proof:** add controls, seeds, replay, flake history, and false-positive resistance.
6. **Agent loop:** structured findings, host hooks, permission policies, fix/retest behavior.
7. **Issue and feedback loop:** GitHub, Linear, support, Sentry, session replay, and incident inputs.
8. **CI/CD guidance:** generate and maintain project-aware workflows and gates.
9. **Deploy rehearsal:** migration, artifact, environment, rollout, rollback, and smoke verification.
10. **Production witness:** read-only observation, safe reproduction, and regression harvesting.
11. **Scale and operations:** remote runners, device farms, larger load, multi-region behavior, and progressive delivery integrations.
12. **Always-on software collaborator:** proactive but transparent, budgeted, permission-controlled, and never silently destructive.

## References

[1]: https://playwright.dev/docs/api-testing "Playwright — API testing"
[2]: https://maestro.dev/ "Maestro — Agentic UI testing for mobile and web"
[3]: https://appium.io/docs/en/2.0/ "Appium — Cross-platform automation documentation"
[4]: https://opentelemetry.io/docs/concepts/semantic-conventions/ "OpenTelemetry — Semantic conventions"
[5]: https://docs.sentry.io/platforms/javascript/session-replay/understanding-sessions/ "Sentry — Understanding browser replay sessions"
[6]: https://code.claude.com/docs/en/permission-modes "Claude Code — Permission modes"
[7]: https://docs.github.com/en/rest/issues/issues "GitHub REST API — Issues"
[8]: https://linear.app/docs/api-and-webhooks "Linear — API and webhooks"
[9]: https://developer.android.com/studio/gemini/agent-mode "Android Studio — Agent Mode"
[10]: https://developer.apple.com/documentation/xcode/giving-external-agents-access-to-xcode "Apple Xcode — Giving external agents access to Xcode"
[11]: https://docs.flutter.dev/ai/get-started "Flutter — Get started developing with AI"
[12]: https://docs.github.com/articles/ignoring-files "GitHub — Ignoring files"
[13]: https://docs.github.com/en/code-security/concepts/secret-security/push-protection "GitHub — Push protection"
[14]: https://12factor.net/config "The Twelve-Factor App — Config"
[15]: https://google.github.io/styleguide/tsguide.html "Google — TypeScript Style Guide"
[16]: https://code.claude.com/docs/en/plugins/overview "Claude Code — Plugins overview"
[17]: https://code.claude.com/docs/en/plugins/manifest-reference "Claude Code — Plugin manifest reference"
[18]: https://code.claude.com/docs/en/skills "Claude Code — Skills"
