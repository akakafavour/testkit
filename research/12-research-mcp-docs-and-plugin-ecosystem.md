# TestKit — Landscape Research B: Docs MCP Servers & the Agent Plugin Ecosystem

*Every entry verified against a primary source (official docs, repo, or registry) during Oct 2026. Items that could not be independently verified are flagged.*

---

## PART 1 — Documentation & Knowledge MCP Servers

### 1.1 Library-documentation grounding (the Context7 category)

**1. Upstash Context7** — `github.com/upstash/context7` · `context7.com` · `https://mcp.context7.com/mcp`
- ~62.5k⭐ · `@upstash/context7-mcp` ~880k weekly downloads, v4.0.4
- **Indexes first-party docs only** — official docs sites, API refs — curated and version-pinned *by the library owners*. No forum threads. **Malware- and prompt-injection-scanned before indexing.** Every result carries library + source URL.
- **Tools:** two-phase. `resolve-library-id(name, query)` → ranked IDs (`/mongodb/docs`); then `query-docs(libraryId, query)` → version-matched docs + snippets. Known ID in prompt skips step 1.
- **New (2026-09-22):** single-call **Search API** — one question in, picks libraries + snippets + reranks. JSON returns per-snippet `codeTokens` so an agent can budget context.
- **Distribution is notable:** ships CLI+Skills mode *and* MCP. `ctx7` CLI installs a skill teaching `ctx7 library <name>` — no MCP server required. Flags `--cursor`, `--claude`, `--opencode`.
- Their claim: *"Scanned before indexing… this matters more for grounding than for anything else, because the tool result goes directly into the model's context."* — **Exactly TestKit's threat model.**

**2. Ref.tools (Ref MCP)** — `ref.tools/mcp` · `github.com/ref-tools/ref-tools-mcp`
- Competes with Context7 on a different bet: **docs search over public AND private docs** (indexes your repos and PDFs).
- Tools: `ref_search_documentation`, `ref_read_url`. Publishes **MCP prompts** (`search_docs`, `my_docs`) so clients autocomplete the prompting.
- **Clever bit to copy:** uses the MCP *session* to track search trajectory — won't return the same result twice, drops irrelevant sections, returns ~5k relevant tokens instead of a whole page.
- Also ships **Ref Plans MCP** with `LaunchAgent`/`SendMessage` multi-agent orchestration.

**3. DeepWiki (Cognition)** — `deepwiki.com` · `github.com/CognitionAI/deepwiki`
- Free, no-auth remote MCP. `ask_question`, `read_wiki_structure`, `read_wiki_contents`.
- Indexes **public GitHub repos** (50k+ at launch, May 2025). Architecture diagrams, docs, source links. Private repos need Devin.
- **Codebase** knowledge, not **library API** knowledge. Good for "how does X actually work"; bad for "what is the current signature of `foo()`."
- Self-hostable: `AsyncFuncAI/deepwiki-open`.

**4. Mintlify Index MCP** — `index.mintlify.com` — free, **no auth** on the public server. Every Mintlify site also gets its own per-domain search MCP. Serves `/llms.txt` + markdown for every page. Also a write-capable admin MCP at `mcp.mintlify.com` (edit pages, update `docs.json`, open PRs) — good precedent for splitting read vs write servers.

**5. Prisma MCP** — `mcp.prisma.io/mcp` · `github.com/prisma/mcp` — `IntrospectSchemaTool`, OAuth-on-first-use. Serves `llms.txt` + `.md`.

**6. Cloudflare docs MCP** — `docs.mcp.cloudflare.com/mcp` — `search_cloudflare_documentation`, `migrate_pages_to_workers_guide`. Plus `github.com/cloudflare/mcp` (**894⭐**) **"Code Mode."**

**7. Google MCP Toolbox for Databases** — `github.com/googleapis/mcp-toolbox` (**16,497⭐**) — connects agents to ~20 databases. **Generates Agent Skills**, ships Gemini CLI extensions + Antigravity MCP Store listing. Template for TestKit's distribution story.

**8. AWS MCP servers** — `github.com/awslabs/mcp` — `aws-api` is **2 tools** (`call_aws`, `suggest_aws_commands`) — same token-conservation bet as Code Mode. ⚠️ `aws-terraform` is **DEPRECATED**.

**9. HashiCorp Terraform MCP** — `github.com/hashicorp/terraform-mcp-server` (~1.5k⭐) — Registry (public+private) + HCP APIs. Ships `instructions.md` as **replaceable default instructions** — a pattern worth stealing (server ships *editable* behavior docs).

### 1.2 Web search / extraction

| # | Name | Stars | Notes |
|---|---|---|---|
| 11 | **Exa MCP** | 4,083 | `web_search_exa`, `web_fetch_exa`, `web_search_advanced_exa`, `agent_run`. **Now ships as an Agent Plugin.** README: *"MCP-only setups still get the tools; **skills add orchestration on top.**"* ← **The TestKit thesis in one line** |
| 12 | **Tavily MCP** | 2,416 | `tavily_search/extract/map/crawl`. ⚠️ May 2026 snapshot classified official repo **STALE**. Acquired by Nebius Feb 2026 |
| 13 | **Brave Search MCP** | 1,478 | Fastest on 5 dev queries (avg 521ms vs Tavily 806ms), highest first-party doc hits in top-5 (40% vs 28%). Only *search-only* server here |
| 14 | **Kagi MCP** | 513 | `kagi_search_fetch`, `kagi_extract`. `kagi_fastgpt`/`kagi_summarizer` removed. Search API in closed beta |
| 15 | **Perplexity MCP** | — | Third-party. Returns *synthesized cited answers* — good for cross-checking, bad for retrieving an API signature |
| 16 | **Firecrawl MCP** | **7,536** | `scrape, search, crawl, map, extract, parse, agent, interact, monitor`. **Keyless hosted tier exposes only 3 tools**; `-search` variant exposes 8. Ships CLI+skill (`npx firecrawl-cli init --all`) + Grok Build plugin |
| 17 | **Firecrawl Toolkit** | — | ⭐ **Architecturally identical to what TestKit wants:** shell-first tool that **writes pages to local Markdown and prints only `true` to stdout**, so the agent reads only relevant sections. Python MCP + Go CLI. Targets Codex, Claude Code, OpenCode |
| 18 | **mcp-omnisearch** | 351 | Unifies Tavily, Brave, Kagi, Exa, Linkup, Firecrawl + GitHub search behind 4 tools. **Answers "why call five servers from inside a scenario"** |

### 1.3 Reader / markdown layer

**19. Jina AI Reader + MCP** — `r.jina.ai` · `mcp.jina.ai/v1` — `primer` (free), `read_url`, `capture_screenshot_url`, `search_web_deep` (search → read each → listwise rerank), `search_arxiv`, `search_ssrn`. Token trick: `?exclude_tags=parallel` removes ~4 tools' worth of context.

**20. Defuddle** — `github.com/kepano/defuddle` — by Obsidian's Steph Ango. Main-content extraction, Readability replacement.
> ⭐ **Most TestKit-relevant thing is the packaging:** kepano ships it as a **skill** — a 41-line `SKILL.md` whose description says *"Prefer over WebFetch for standard web pages — it removes navigation, ads, and clutter, reducing token usage."* **Zero MCP, one skill, one CLI.**

**21. llms.txt** — `llmstxt.org` · `github.com/AnswerDotAI/llms-txt` (2,629⭐) — Jeremy Howard, Sept 2024; **v2 Aug 10 2026**. v2's change: drop `llms_txt2ctx`, tell agents to *view/search llms.txt then follow links*.
- Adoption: OpenAI, Anthropic, Google each publish one. Plus Stripe, Vercel, Cloudflare, Perplexity, LangChain, HuggingFace. **Mintlify auto-generates across 8,000+ hosted docs sites.** GitBook, Yoast, Wix too. **Chrome Lighthouse audits for it** as part of agentic-browsing checks.
- `server-llm-txt` MCP exists for fetch+search.
- ⭐ **Zero-cost, zero-API-key, zero-vendor-lock way to ground a scenario in real docs.** Should be tier 0.

**22. GitHub MCP Server** — `github.com/github/github-mcp-server` (**33,295⭐**) — per-host install guides for **Claude, Codex, Cursor, OpenCode, Windsurf, Zed, Rovo Dev CLI, VS Code**. Best single reference for "what does each host's MCP config look like."

### 1.4 Code-search / context-packing

**23. Serena** — `github.com/oraios/serena` (**29,901⭐**) — "The IDE for your coding agent." Symbol-level retrieval via **LSP** (40+ languages). `find_symbol`, `replace_symbol_body`, `safe_delete`, `rename`, type hierarchy. **New in v2: `serena_repl`** executes Python against the API so many round-trips collapse into one call. Same thesis as Cloudflare Code Mode.

**24. Sourcegraph** — Deep Search + Code Search + SCIP-powered MCP.
**25. ast-grep MCP** — `github.com/ast-grep/ast-grep-mcp` (469⭐) — structural search. `dump_syntax_tree` + `ast-grep`.
**26. Playwright MCP** — `github.com/microsoft/playwright-mcp` (**36,810⭐**) — accessibility-tree-driven, no vision model.
> ⚠️ **Critical for TestKit:** Microsoft's own README now argues *against* MCP for coding agents — *"Modern coding agents increasingly favor CLI-based workflows exposed as SKILLs over MCP because CLI invocations are more token-efficient: they avoid loading large tool schemas and verbose accessibility trees."* MCP retained only for exploratory/self-healing/iterative loops. **~1.5k tokens of context per load.**

### 1.5 Vendor MCP servers

| # | Name | What it gives you |
|---|---|---|
| 27 | **Stripe MCP + agent toolkit** | `stripe_api_search/details/read/write`, `search_stripe_documentation`, `stripe_implementation_planner`. ⭐ **Best-in-class distribution:** `stripe agent setup` auto-detects agents, installs MCP + skills, self-updates. Machine-readable catalog at `docs.stripe.com/.well-known/skills/index.json`. Every docs URL serves markdown via `.md` suffix |
| 28 | **Supabase MCP** | URL-scoped (`?project_ref=…&read_only=true&features=…`). Ships **agent skills** + a Plugin for one-step setup |
| 29 | **Vercel MCP** | `npx add-mcp https://mcp.vercel.com` auto-configures every installed client. Also CLI workflows, **Skills**, `llms.txt`, `sitemap.md` |
| 30 | **Linear MCP** | Authenticated remote spec, OAuth 2.1 + DCR. `/mcp/readonly` endpoint |
| 31 | **Notion MCP** | Recommends remote server; the OSS `notion-mcp-server` is **no longer maintained** |
| 32 | **Sentry MCP** | Issue/event triage. `search_events`/`search_issues` need an LLM provider; rest works without |
| 33 | **Cloudflare catalog** | 17 servers incl. browser rendering (fetch → markdown + screenshot), container sandbox, AutoRAG |
| 34 | **Docker MCP Catalog** | `hub.docker.com/mcp` — signed, health-scored, config-declared. De-facto trust surface alongside the official registry |

### 1.6 MCP security & supply chain

**36. mcp-scan (Invariant Labs)** — `github.com/invariantlabs-ai/mcp-scan` — Apache-2.0. `uvx mcp-scan@latest`. **Static scan** + **runtime proxy with guardrails**. Detects prompt injection in tool descriptions, tool poisoning, **rug pulls (hash-based tool pinning)**, cross-origin escalation/tool shadowing, secrets in tool definitions. Auto-discovers Claude/Cursor/Windsurf configs.

**37. Invariant Guardrails** — the rule engine behind mcp-scan.
**38. Cisco AI Defense** — MCP runtime guardrails, **MCP Catalog**, **AI Bill of Materials**, 200+ threat subcategories. Maps to NIST, OWASP, MITRE.
**39. VerifyMCP** — `verifymcp.io` — independent scanner, scores 0–100 on supply-chain security, schema quality/AI-usability, capabilities. **Checks whether a tool's `description` text tries to manipulate the model.** ⭐ Model for TestKit's own posture.
**40. Enkrypt AI** — Scan → Curate → Configure → Enforce, with decision receipts.

### 1.7 MCP tooling

**MCP Inspector** (now one package: Web/CLI/TUI) · **TS SDK v2** · **Python SDK** (`mcp`, FastMCP) · **mcp-use** · **Official Registry** (`mcp/<namespace>/<name>` slugs) · **VoltAgent/official-mcp-servers** (**280+ curated**) · **awesome-mcp** · **Spec revision: 2026-07-28** (`/sse` now alias-only)

---

## PART 2 — The Agent Plugin Ecosystem

### 2.1 Claude Code (the reference implementation)

```
<plugin-root>/
├── .claude-plugin/plugin.json     # optional manifest
├── skills/<name>/SKILL.md         # one dir per skill
├── commands/*.md                  # flat markdown
├── agents/*.md                    # subagent definitions
├── hooks/hooks.json               # merged with user/project hooks
├── output-styles/
├── .mcp.json
└── workflows/
```

**Constraints that shape a TestKit design:**
- **Custom commands have been merged into skills.** Both `.claude/commands/deploy.md` and `.claude/skills/deploy/SKILL.md` produce `/deploy`; skills are the recommended successor.
- Plugin MCP servers run in **every session** where enabled. `${CLAUDE_PLUGIN_ROOT}` + `${user_config.*}` substitution (exec form only — shell form fails). `user_config` is how plugins prompt for config.
- ⚠️ **Token tax is explicit:** *"for each skill, agent, and command that Claude can invoke on its own, the name and description are in Claude's context on every turn… those tokens count toward your usage."* **Budget your skill descriptions.**
- Hooks can live in **skill and subagent frontmatter**, not just settings. `once: true` removes after first success. Handlers: `command`, `http`, `prompt`, `mcp_tool`.
- Events: `PreToolUse`, `PostToolUse`, `PostToolBatch`, `SessionStart/End`, `UserPromptSubmit`, `Stop`, `SubagentStop`, `PreCompact`, `Notification`, `ConfigChange`, `CwdChanged`, `PermissionRequest`, `ExitPlanMode`, `TaskCreated/Completed`, `TeammateIdle`, `WorktreeCreate/Remove`, `Pre/PostModelSwitch`, `Setup`, `MessageDisplay`.
- Subagent frontmatter: `description, tools, disallowedTools, model, permissionMode, mcpServers, hooks, maxTurns, skills, initialPrompt, memory, effort, background, omitClaudeMd, isolation`.
- Enterprise: `allowManagedHooksOnly` blocks user/project/plugin hooks — **except plugins force-enabled via managed `enabledPlugins`.** An admin can distribute vetted hooks through an org marketplace. ⭐ That's TestKit's enterprise story.

### 2.2 Codex CLI
`~/.codex/config.toml` + `.codex/config.toml`. **Shared** by ChatGPT desktop app, Codex CLI, IDE extension.
- Rich per-server controls: `enabled_tools`, `disabled_tools`, `default_tools_approval_mode` (`auto|prompt|writes|approve`), `tools.<tool>.approval_mode`, `output_token_limit`, `http_headers_helper`, OAuth w/ `codex mcp login`.
- **AGENTS.md chain:** `~/.codex/AGENTS.override.md` → `~/.codex/AGENTS.md` → per-level overrides from git root to cwd.
- **Subagents:** `.codex/agents/*.toml` with `name, description, model, model_reasoning_effort, sandbox_mode, developer_instructions, mcpServers, skills.config`.
- **Sandbox:** `[sandbox] permission_profile = ":read-only" | ":workspace" (default) | ":danger-full-access"`, `[sandbox.network] allowed_domains`. `trust_level = "untrusted"` makes Codex ignore project config entirely.

### 2.3 Gemini CLI / Antigravity
⚠️ **Gemini CLI was replaced by Antigravity CLI on June 18, 2026** for unpaid tier and Google One users. Treat as **two hosts**.
Extensions now package **prompts + MCP servers + custom commands + themes + hooks + sub-agents + agent skills** — the most complete single-package surface.
Layout: `~/.gemini/extensions/<name>/` with `gemini-extension.json`, `commands/*.toml` (namespaced → `/gcs:sync`), `GEMINI.md`, `hooks/hooks.json`, `policies/*.toml`, `mcp.json`. `${extensionPath}` substitution. Commands are TOML with `{{args}}` and `!{shell}`.
**GEMINI.md can be pointed at AGENTS.md:** `{"context": {"fileName": ["AGENTS.md","CONTEXT.md","GEMINI.md"]}}`.

### 2.4 Cursor
Customize surface is now **Plugins, Rules, Skills, Subagents, Hooks, MCP** in one place. `.cursor/hooks.json` `{"version":1,"hooks":{...}}`.
Events: `beforeShellExecution`, `afterShellExecution`, `beforeMCPExecution`, `afterMCPExecution`, `beforeReadFile`, `afterFileEdit`, `beforeSubmitPrompt`, `afterAgentResponse`, `afterAgentThought`, `stop`, `subagentStart`, `preToolUse`/`postToolUse`, `beforeTabFileRead`, `afterTabFileEdit`, `workspaceOpen`.
**Two hook types:** command-based (default; stdin JSON → stdout JSON; **exit 2 = block**) and **prompt-based (LLM-evaluated)**. Cloud agents run command hooks only.
Enterprise MDM paths per-OS. Background Agents + **Bugbot** are first-class.

### 2.5 opencode
**Best skills discovery in the ecosystem:** `.opencode/skills/`, `.claude/skills/`, `.agents/skills/` (project, walking up to git worktree) + global equivalents. ⭐ **`.agents/skills/` is the cross-client standard path.**
V2 accepts a `skills` array in `opencode.json` pointing at local dirs **or HTTP catalogs** — a skills registry can be consumed as a URL.
**Plugins** extend in-process: `ctx.agent`, `ctx.command`, `ctx.skill`, `ctx.tool`, `ctx.session`, `ctx.integration`, `ctx.reference`, `ctx.catalog`. Transform hooks + `ctx.tool.hook("execute.before"/"after")`. V2 beta; ordering not guaranteed.

### 2.6 ⭐ Cross-cutting standards

**50. Agent Skills (open format)** — `agentskills.io` · `github.com/agentskills/agentskills` (**25,786⭐**, Apache-2.0)
- Originally Anthropic, released open Dec 2025.
- Layout: `skill-name/SKILL.md` (required), `scripts/`, `references/`, `assets/`.
- Frontmatter: `name` (must match directory), `description` (what + when + triggers), `license`, `compatibility`, `metadata`, `allowed-tools`.
- **Progressive disclosure:** (1) startup loads only name + description (~50–100 tokens each); (2) on match, load the body; (3) load supporting files as referenced.
- Discovery: `.agents/skills/` (highest priority, cross-client standard), `.claude/skills/`, `~/.agents/skills/`, `~/.<client>/skills/`.

**51. ⭐⭐ Agent Plugins Specification 1.0.0** — `agent-plugins.org` · `github.com/agentplugins/agent-plugins-spec` (1,335⭐)
- **Launched Aug 6, 2026** as the open cross-tool specification. **VS Code adopted it** alongside Copilot CLI. TSC includes maintainers from **Amazon, Cursor, Microsoft, OpenAI, Vercel**.
- **v1 defines exactly two component types: skills and MCP servers.** Clients MUST ignore what they don't support.
```
my-plugin/
├── plugin.json                     # REQUIRED
├── skills/<skill-name>/SKILL.md
├── mcp.json                        # stdio, Streamable HTTP, or legacy HTTP+SSE
└── com.example.client/             # client-namespaced extension (hooks, agents)
```
- Distribution, installation, permissions, UX **left to each client**. It defines an interoperability *floor*, not a ceiling.
- **This is the answer to "the de-facto standard shape of an agent plugin."**

**52. AGENTS.md** — `agents.md` — Linux Foundation **Agentic AI Foundation** (Dec 2025) alongside MCP and Goose. 60k+ OSS projects, 28+ tools. Native in Codex, Copilot, Cursor, Windsurf, Amp, Devin, Aider, Zed, Jules, VS Code, Junie, Roo Code, Warp, Factory, opencode — **and Claude Code now reads it too.** Gemini CLI is the holdout but can be pointed at it.

**53. A2A Protocol** — `a2a-protocol.org` — Google → Linux Foundation. 22k⭐, 150+ orgs. Signed Agent Cards. **Explicitly complementary: MCP = agent→tool, A2A = agent→agent.** A2A does *not* specify sub-agent invocation.

**54. MCP spec: 2026-07-28.**

### 2.7 Hook systems compared

| Host | Config | Handler types | Notes |
|---|---|---|---|
| Claude Code | `settings.json`, plugin `hooks/hooks.json`, frontmatter | command, http, prompt, mcp_tool | Richest events; `once`; `${CLAUDE_PLUGIN_ROOT}`; managed-policy layer |
| Cursor | `.cursor/hooks.json` | command, **prompt (LLM-evaluated)** | 4 tiers; cloud = command only |
| Gemini CLI | extension `hooks/hooks.json` | command + prompts | — |
| opencode | JS/TS plugin `ctx.tool.hook(...)` | arbitrary code | Transform + runtime; in-process |
| Codex CLI | `config.toml` | command | Pre/PostToolUse |

### 2.8 Other hosts

| Host | Config | Skills | Hooks | Note |
|---|---|---|---|---|
| **GitHub Copilot** | `.github/{instructions,prompts,agents,skills,hooks}/`, `AGENTS.md`, `.github/mcp.json` | ✅ 423 skills in awesome-copilot | ✅ | **Adopted Agent Plugins 1.0 Aug 6 2026.** `copilot plugin install <name>@awesome-copilot` |
| **Cline** | `.cline/{rules,hooks,skills}/` | ✅ | ✅ | Reads AGENTS.md |
| **Roo Code** | `.roo/` | ✅ | — | ⚠️ **Archived May 2026** → became Kilo Code |
| **Kilo Code** | `.kilo/{agents,rules,command}`, `kilo.jsonc` | ✅ | ✅ | Apache-2.0, BYOK, 500+ models. Acquired by Anaconda 2026 |
| **Windsurf** | `.windsurfrules` | ✅ | ✅ | Now Cognition's Devin Desktop; `.devin/hooks.v1.json` |
| **Zed** | `.zed/settings.json`, `context_servers` | — | — | AGENTS.md yes; weak otherwise |
| **Aider** | `.aider.conf.yml`, `CONVENTIONS.md` | — | — | **MCP not in official docs** — AGENTS.md-only |
| **Goose** | `.goosehints` | ✅ | — | MCP is a core design point |
| **Amazon Q / Kiro** | `~/.kiro/settings/mcp.json` | — | — | MCP client + server |

### 2.9 Distribution

**55. skills.sh** — `npx skills add <owner>/<repo>` installs to **all detected agents — 75+ agents**. `-a <agent>`, `-g`/`-p`, `--all`, `skills find/check/update`. Canonical repo is **`vercel-labs/skills`**. Leaderboard + **security audits** (`/api/v1/skills/audit/...`). ⚠️ Fork fragmentation is real.

**56. awesome-copilot** — 222 agents, 194 instructions, **423 skills**, 168 plugins. Ships `llms.txt`. Has a `microsoft-docs` plugin — closest precedent to a TestKit docs plugin.
**57. hesreallyhim/awesome-claude-code** (~36.8k⭐) · **58. quemsah/awesome-claude-plugins** (15,134 plugin repos indexed by May 2026) · **59. affaan-m/everything-claude-code** (~141k⭐)

### 2.10 ⭐ Precedent: verification/QA plugins shipped this way

- **Stripe** ⭐ best distribution precedent: `stripe agent setup` detects agents, installs MCP + skills, self-updates, publishes a machine-readable skills catalog.
- **Supabase** — MCP + agent skills in one install.
- **Anthropic's security-guidance plugin** — the canonical "verify, don't just advise" hook pattern (separate model review fed back into the session).
- **Playwright CLI + SKILLS** — Microsoft explicitly shipping a *skill-based* browser-QA surface instead of MCP.
- **TestSprite** (commercial) — PRD-or-code → Playwright tests in cloud sandboxes → pinpoint feedback to the agent via MCP. Closest commercial analogue.
- **VerifyMCP catalog** lists *Vigilis* ("Gate AI-written tests: heal safe drift, refuse to hide real bugs, **sign a verifiable receipt**"), *five46* (BYOK local agent that tests your app and writes a real Playwright spec), *comber*. ⭐ **Verifiable receipts for AI-generated tests is a live, unsolved niche with no category leader.**

---

## Synthesis

### 4.1 ⭐ The de-facto standard shape of a plugin TODAY

As of Oct 2026 the converged shape is **Agent Plugins 1.0** (`agent-plugins.org`), which Claude Code, Copilot, Exa, Stripe, and Firecrawl all satisfy as a strict subset.

```
testkit/
├── plugin.json                    # {"$schema":".../plugin.schema.json","name":"testkit"}
├── mcp.json                       # TestKit's MCP server: stdio + Streamable HTTP
├── skills/
│   ├── <skill-name>/
│   │   ├── SKILL.md               # name + description + body
│   │   ├── scripts/               # executable helpers
│   │   └── references/            # long docs, loaded on demand
├── AGENTS.md                      # portable project-level instructions
└── hosts/
    ├── claude-code/               # .claude-plugin/plugin.json + hooks/hooks.json + agents/
    ├── gemini/                    # gemini-extension.json + commands/*.toml + hooks/hooks.json
    ├── cursor/                    # hooks.json (version:1) + rules/*.mdc
    ├── codex/                     # config.toml + .codex/agents/*.toml
    └── opencode/                  # plugin .ts transform/hook registrations
```

**Rules now effectively universal:**
1. **`plugin.json` at root is the identity.** `{"name":"testkit"}` + optional `$schema`.
2. **`skills/<name>/SKILL.md` — no deeper nesting.** Frontmatter carries only spec fields. Host-specific fields go in the overlay, not the portable core.
3. **`name` must equal the directory name.** Free win for cross-host correctness.
4. **`mcp.json` at root** — stdio, Streamable HTTP, or legacy. TestKit's "call docs MCPs from inside a scenario" story runs through this one file.
5. **Host extras under `com.<vendor>.<client>/`.** Agent Plugins 1.0 reserves this explicitly.
6. **Only name + description cost tokens at startup** (~50–100 each). **Write descriptions for *triggering*, not for reading.**

> ⭐ **The plugin's MCP server is the portable core. The skills are the orchestration. The host overlay is ~10% of the code and ~0% of the value.**

### 4.2 ⭐ The 5 best docs/knowledge MCP servers to call from inside a scenario

Ranked for TestKit's need — *grounding generated scenarios in real, current library docs rather than model memory*:

1. **Context7** — default. First-party, owner-curated, version-pinned, injection-scanned, source-attributed. Single-call Search API. **Fallback tier: `npx ctx7 setup` CLI+skill** where you don't want a running server.
2. **Ref MCP** — the complement. Covers *private* docs (your repos, your PDFs). Session-aware: no duplicate results, ~5k relevant tokens instead of a whole page.
3. **DeepWiki** — for *behavioral* correctness. Library docs say what the API should look like; DeepWiki says how the thing actually behaves. Zero auth, zero cost.
4. **Firecrawl** — the escape hatch. Keyless scrape/search/parse. When Context7 has nothing (niche internal SDK, a blog post that is the only spec, a changelog), this gets raw bytes as markdown including PDFs and JS-rendered pages.
5. **llms.txt direct fetch** — `https://<docs-host>/llms.txt` — zero API key, zero rate limit, zero vendor lock. **This should be tier 0 — try it before spending a Context7 call.**

Runner-up: **github-mcp-server** (33.3k⭐) for grounding in actual source when docs and behavior disagree.

### 4.3 Who has real first-class skills/hooks/subagents vs. MCP-only

**Tier 1 — full first-class (build TestKit natively here):**
- **Claude Code** — everything + marketplaces + `${CLAUDE_PLUGIN_ROOT}` + output styles + agent teams. Reference implementation.
- **Cursor** — plugins, rules, skills, subagents, hooks (incl. LLM-evaluated prompt hooks), MCP, Bugbot, background agents.
- **opencode** — best skills discovery (reads `.opencode/`, `.claude/`, `.agents/`), commands, agents, in-process transform + runtime hooks.
- **Gemini CLI / Antigravity** — most complete *extension* surface. But **Antigravity replaced Gemini CLI June 18 2026** — treat as two hosts.
- **GitHub Copilot** — 423 skills, 168 plugins, hooks, **Agent Plugins 1.0 native since Aug 6 2026**. Tier 1 for distribution.

**Tier 2 — real skills, partial hooks/subagents:** Codex CLI (skills + TOML subagents, hooks are config.toml commands), Kilo Code, Cline, Windsurf/Devin Desktop.

**Tier 3 — MCP-only or MCP + plain markdown:** Zed, Aider (**MCP not in official docs**), Goose, Amazon Q/Kiro, Continue.

### 4.4 Avoiding six different integrations

**The minimal portable core (write once):**
```
plugin.json            # Agent Plugins 1.0
skills/*/SKILL.md      # Agent Skills spec — pure frontmatter
mcp.json               # stdio + Streamable HTTP
AGENTS.md              # project-level instructions
scripts/               # the actual work (language-agnostic CLI)
```
> Every item is a published, ratified, cross-vendor standard. A skill written this way already loads in Claude Code, Cursor, opencode, Codex, Copilot, Cline, Kilo, Windsurf, and 60+ others with **zero modification** — only the install path differs.

**What must be per-host (thin adapters only):**

| Concern | Claude Code | Gemini | Cursor | Codex | opencode | Copilot |
|---|---|---|---|---|---|---|
| Manifest | `.claude-plugin/plugin.json` | `gemini-extension.json` | (none needed) | n/a | `opencode.json` | `plugin.json` |
| Hooks | `hooks/hooks.json` (command/http/prompt/mcp_tool) | `hooks/hooks.json` | `hooks.json` v1 (command+prompt) | config.toml | JS `ctx.tool.hook` | `.github/hooks/` |
| Commands | `commands/*.md` (or skills) | `commands/*.toml` | — | — | `ctx.command.transform` | `.github/prompts/` |
| Subagents | `agents/*.md` | `.gemini/agents/` | `.cursor/agents/` | `.codex/agents/*.toml` | `ctx.agent.transform` | `.github/agents/` |
| Config path | `.mcp.json` | manifest `mcpServers` | `~/.cursor/mcp.json` | `config.toml` | `opencode.json` | `.github/mcp.json` |

**Practical strategy:**
1. **Ship the core as an Agent Plugin 1.0 package first.** Covers Claude Code, Copilot, and every spec-conformant client.
2. **Ship an `npx testkit install` CLI doing per-host placement** — what skills.sh does with `-a <agent>`. Table-driven config, ~200 lines, not six products.
3. **Write behavioral logic once, in `scripts/`**; every SKILL.md and hook is a thin shim. **CLI + skill beats MCP for token cost; MCP beats CLI for stateful loops.** Playwright's own README argues for this split.
4. **Hooks are the one thing you cannot abstract.** Budget 3 shims max (Claude, Cursor, Gemini), each ~30 lines, each calling the same script. Note the asymmetry: Cursor + Claude have LLM prompt hooks; Codex is command-only. **A gate that works identically everywhere should be a command, not a prompt.**
5. **Don't build six marketplace submissions before one working plugin.** Stripe, Supabase, Exa, Firecrawl all shipped spec-conformant packages first.

### 4.5 Is there precedent for a verification/QA plugin? 

**Closest existing:** none is a category leader.
- Vendor MCP servers carrying a verification role: Stripe's `stripe_implementation_planner`, Supabase's RLS advisory, Sentry's triage.
- Anthropic's **security-guidance plugin** — the canonical "verify, don't just advise" hook pattern.
- AI-generated-test gating is a **crowded-but-unclaimed niche** (Vigilis, five46, comber, TestSprite).
- **What nobody has done:** a plugin that (a) *generates* scenarios grounded in live docs MCPs, (b) *executes* them, (c) *reconciles* failures against current docs, and (d) emits a **signed, portable verification receipt** — across seven hosts via one `plugin.json`.

> ⭐ **That gap is TestKit's opening.** Distribution is solved (Agent Plugins 1.0 + Agent Skills + `npx skills add` + 75+ agents). Credibility — grounding scenarios in real current docs rather than model memory — is what Context7/Ref/DeepWiki/Firecrawl/llms.txt make possible. **Combine them and you have the first verification plugin whose scenarios are provably doc-grounded and whose results are provably reproducible.**

> ⚠️ **Design warning from the evidence:** Stripe instructs users to *"Enable human confirmation of tools and exercise caution when using the Stripe MCP with other servers to avoid prompt injection attacks."* TestKit will orchestrate five third-party MCP servers inside a scenario generator. **Treat every docs-MCP result as untrusted input, attribute every fact to a source URL, and never let a retrieved doc string trigger a tool call without going back through your own skill instructions.**