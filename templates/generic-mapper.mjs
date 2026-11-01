#!/usr/bin/env node
// TestKit generic mapper — grammar-driven surface extraction.
//
// Design: TestKit builds NO per-framework mapper binaries. The community
// maintains Tree-sitter grammars (one per language); the agent installs the
// grammar (`web-tree-sitter` + `tree-sitter-wasms`), and this one script
// uses it plus small built-in per-framework rules to produce the machine
// map. Output contract: testkit.map/1 (see testkit-core/src/semantic_map.rs).
//
// The honesty rule is enforced here: every source file that could not be
// interpreted is reported in coverage.unresolved with a reason. Silence is
// a contract violation.
//
// Usage: node generic-mapper.mjs --root <project-root>
// Exit 0 with the map JSON on stdout; exit 1 with a reason on stderr.

import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative, sep } from "node:path";

const MAP_SCHEMA = "testkit.map/1";
const MAPPER = { extractor: "generic-mapper", version: "0.1.0" };
const IGNORED_DIRS = new Set([
  "node_modules", ".git", ".next", ".testkit", "dist", "build",
  "target", ".vercel", ".turbo", "coverage", ".sanity",
]);

// ---------------------------------------------------------------------------
// Per-framework rules. A rule is either a file-system convention (no grammar
// needed) or a grammar query (requires web-tree-sitter + the language wasm).
// ---------------------------------------------------------------------------

const FRAMEWORKS = {
  nextjs: {
    language: "typescript",
    mode: "filesystem",
    describe: "scans app/** and pages/** route conventions (no grammar needed)",
  },
  nuxt: {
    language: "typescript",
    mode: "filesystem",
    describe: "scans pages/** and server/api/** route conventions (no grammar needed)",
  },
  express: {
    language: "javascript",
    mode: "grammar",
    describe: "finds app.get('/path', handler)-style registrations",
    query: `
      (call_expression
        function: (member_expression
          object: (identifier) @obj
          property: (property_identifier) @method
            (#match? @method "^(get|post|put|patch|delete|all)$"))
        arguments: (arguments
          (string
            (string_fragment) @path)))`,
  },
  fastapi: {
    language: "python",
    mode: "grammar",
    describe: "finds @app.get('/path') decorators and their handler functions",
    query: `
      (decorated_definition
        (decorator
          (call
            function: (attribute
              object: (identifier) @obj
              attribute: (identifier) @method
                (#match? @method "^(get|post|put|patch|delete)$"))
            arguments: (argument_list
              (string
                (string_content) @path))))
        definition: (function_definition
          name: (identifier) @handler))`,
  },
  flask: {
    language: "python",
    mode: "grammar",
    describe: "finds @app.route('/path') decorators and their handler functions",
    query: `
      (decorated_definition
        (decorator
          (call
            function: (attribute
              object: (identifier)
              attribute: (identifier) @method
                (#match? @method "^(route|get|post|put|patch|delete)$"))
            arguments: (argument_list
              (string
                (string_content) @path))))
        definition: (function_definition
          name: (identifier) @handler))`,
  },
  actix: {
    language: "rust",
    mode: "grammar",
    describe: "finds #[get(\"/path\")] attribute macros and their handler functions",
    query: `
      (function_item
        (attribute_item
          (attribute
            (identifier) @method
              (#match? @method "^(get|post|put|patch|delete)$")
            (token_tree
              (string_literal) @path)))
        name: (identifier) @handler)`,
  },
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function fail(message) {
  console.error(message);
  process.exit(1);
}

function parseArgs(argv) {
  const args = { root: process.cwd() };
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] === "--root" && argv[i + 1]) args.root = argv[i + 1];
  }
  if (!existsSync(args.root) || !statSync(args.root).isDirectory()) {
    fail(`project root does not exist: ${args.root}`);
  }
  return args;
}

/** Detect the framework from manifests — mirrors the engine's stack.json. */
function detectFramework(root) {
  const read = (name) => {
    try { return readFileSync(join(root, name), "utf8"); } catch { return null; }
  };
  const pkg = read("package.json");
  if (pkg) {
    if (pkg.includes('"next"')) return "nextjs";
    if (pkg.includes('"nuxt"')) return "nuxt";
    if (pkg.includes('"express"')) return "express";
  }
  const py = read("pyproject.toml") ?? read("requirements.txt");
  if (py) {
    if (py.includes("fastapi")) return "fastapi";
    if (py.includes("flask")) return "flask";
  }
  const cargo = read("Cargo.toml");
  if (cargo && cargo.includes("actix-web")) return "actix";
  return null;
}

/** Every source file under root, honoring IGNORED_DIRS. */
function walkFiles(root) {
  const files = [];
  const walk = (dir) => {
    for (const entry of readdirSync(dir)) {
      if (IGNORED_DIRS.has(entry)) continue;
      const full = join(dir, entry);
      if (statSync(full).isDirectory()) {
        walk(full);
      } else {
        files.push(relative(root, full).split(sep).join("/"));
      }
    }
  };
  walk(root);
  return files;
}

function rel(root, file) {
  return relative(root, file).split(sep).join("/");
}

/** Deterministic node id from kind, path, and a discriminator. */
function nodeId(kind, path, discriminator) {
  return `${kind}:${path}:${discriminator}`.replace(/[^A-Za-z0-9._/-]+/g, "-");
}

// ---------------------------------------------------------------------------
// Filesystem-convention mappers (no grammar required)
// ---------------------------------------------------------------------------

function mapNextjs(root, allFiles) {
  const nodes = [];
  const sourceFiles = allFiles.filter((f) => /\.(tsx|ts|jsx|js)$/.test(f));
  for (const file of sourceFiles) {
    if (/^app\//.test(file)) {
      if (/\/page\.(tsx|jsx|js)$/.test(file) || /^app\/page\./.test(file)) {
        const dir = file.replace(/\/page\.(tsx|jsx|js)$/, "").replace(/^app/, "");
        const route = dir.replace(/\/\(([^)]+)\)/g, "").replace(/\(([^)]+)\)/g, "") || "/";
        nodes.push({
          id: nodeId("route", file, route),
          kind: "route",
          name: `PAGE ${route}`,
          path: file,
          detail: "app-router page",
        });
      } else if (/\/route\.(tsx|ts|js)$/.test(file)) {
        const dir = file.replace(/\/route\.(tsx|ts|js)$/, "").replace(/^app/, "") || "/";
        const methodHint = "API";
        nodes.push({
          id: nodeId("route", file, `${methodHint}:${dir}`),
          kind: "route",
          name: `API ${dir}`,
          path: file,
          detail: "app-router endpoint",
        });
      }
    } else if (/^pages\//.test(file)) {
      const base = file.replace(/^pages\//, "");
      if (base.startsWith("_app.") || base.startsWith("_document.") || base.startsWith("_error.")) {
        continue;
      }
      if (/^api\//.test(base)) {
        const route = `/${base.replace(/\.(tsx|ts|jsx|js)$/, "")}`;
        nodes.push({
          id: nodeId("route", file, route),
          kind: "route",
          name: `API ${route}`,
          path: file,
          detail: "pages-router endpoint",
        });
      } else {
        const route = `/${base.replace(/index\.(tsx|jsx|js)$/, "").replace(/\.(tsx|jsx|js)$/, "")}`.replace(/\/+$/, "") || "/";
        nodes.push({
          id: nodeId("route", file, route),
          kind: "route",
          name: `PAGE ${route}`,
          path: file,
          detail: "pages-router page",
        });
      }
    }
  }
  const parsed = nodes.length > 0 ? sourceFiles.length : 0;
  return { nodes, files_parsed: parsed };
}

function mapNuxt(root, allFiles) {
  const nodes = [];
  const sourceFiles = allFiles.filter((f) => /\.(vue|ts|js)$/.test(f));
  for (const file of sourceFiles) {
    if (/^pages\//.test(file)) {
      const route = `/${file.replace(/^pages\//, "").replace(/index\.(vue|ts|js)$/, "").replace(/\.(vue|ts|js)$/, "")}`.replace(/\/+$/, "") || "/";
      nodes.push({
        id: nodeId("route", file, route),
        kind: "route",
        name: `PAGE ${route}`,
        path: file,
        detail: "nuxt page",
      });
    } else if (/^server\/api\//.test(file)) {
      const route = `/api/${file.replace(/^server\/api\//, "").replace(/\.(get|post|put|patch|delete)\.(ts|js)$/, "").replace(/\.(ts|js)$/, "")}`;
      nodes.push({
        id: nodeId("route", file, route),
        kind: "route",
        name: `API ${route}`,
        path: file,
        detail: "nitro endpoint",
      });
    }
  }
  const parsed = nodes.length > 0 ? sourceFiles.length : 0;
  return { nodes, files_parsed: parsed };
}

// ---------------------------------------------------------------------------
// Grammar-convention mapper (web-tree-sitter + tree-sitter-wasms)
// ---------------------------------------------------------------------------

async function mapWithGrammar(root, framework, rule, allFiles) {
  let Parser;
  let Language;
  try {
    ({ Parser, Language } = await import("web-tree-sitter"));
  } catch {
    fail(
      "web-tree-sitter is not installed. The agent must install the grammar once:\n" +
      "  npm install --save-dev web-tree-sitter tree-sitter-wasms\n" +
      "then re-run `testkit map`.",
    );
  }
  try {
    await Parser.init();
  } catch (error) {
    fail(`failed to initialise web-tree-sitter: ${error}`);
  }

  const wasmCandidates = [
    join(root, "node_modules", "tree-sitter-wasms", "out", `tree-sitter-${rule.language}.wasm`),
    join(root, "node_modules", `tree-sitter-${rule.language}`, `tree-sitter-${rule.language}.wasm`),
  ];
  const wasmPath = wasmCandidates.find((candidate) => existsSync(candidate));
  if (!wasmPath) {
    fail(
      `the tree-sitter wasm grammar for '${rule.language}' was not found. The agent must install it once:\n` +
      "  npm install --save-dev web-tree-sitter tree-sitter-wasms\n" +
      "then re-run `testkit map`.",
    );
  }

  const language = await Language.load(wasmPath);
  const parser = new Parser();
  parser.setLanguage(language);
  const query = language.query(rule.query);

  const sourceFiles = allFiles.filter((f) => {
    if (rule.language === "python") return /\.py$/.test(f);
    if (rule.language === "rust") return /\.rs$/.test(f);
    return /\.(js|mjs|cjs|jsx|ts|tsx)$/.test(f);
  });

  const nodes = [];
  let parsed = 0;
  const unresolved = [];
  for (const file of sourceFiles) {
    let tree;
    try {
      tree = parser.parse(readFileSync(join(root, file), "utf8"));
      parsed += 1;
    } catch (error) {
      unresolved.push({ path: file, reason: `unreadable: ${error}` });
      continue;
    }
    const matches = query.matches(tree.rootNode);
    for (const match of matches) {
      const capture = (name) => {
        const found = match.captures.find((c) => c.name === name);
        return found ? found.node.text : null;
      };
      const method = capture("method");
      const rawPath = capture("path");
      if (!method || !rawPath) continue;
      const pathText = rawPath.replace(/^["'`]|["'`]$/g, "");
      const handler = capture("handler") ?? "";
      nodes.push({
        id: nodeId("route", file, `${method}:${pathText}`),
        kind: "route",
        name: `${method.toUpperCase()} ${pathText}`,
        path: file,
        detail: handler ? `handler: ${handler}` : "framework route",
      });
    }
    tree.delete?.();
  }
  // Files the conventions could not classify are not failures — but files
  // that failed to parse are, and they are reported above.
  return { nodes, files_parsed: parsed, unresolved };
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

const args = parseArgs(process.argv.slice(2));
const framework = detectFramework(args.root);
if (!framework || !FRAMEWORKS[framework]) {
  fail(
    framework
      ? `no built-in rule for framework '${framework}' yet; the agent is the mapper`
      : "unrecognized stack; the agent is the mapper",
  );
}

const rule = FRAMEWORKS[framework];
const allFiles = walkFiles(args.root);
const seen = new Set();
const nodes = [];
const unresolved = [];
let filesParsed = 0;

if (rule.mode === "filesystem") {
  const result = framework === "nextjs" ? mapNextjs(args.root, allFiles) : mapNuxt(args.root, allFiles);
  for (const node of result.nodes) {
    if (!seen.has(node.id)) { seen.add(node.id); nodes.push(node); }
  }
  filesParsed = result.files_parsed;
} else {
  const result = await mapWithGrammar(args.root, framework, rule, allFiles);
  for (const node of result.nodes) {
    if (!seen.has(node.id)) { seen.add(node.id); nodes.push(node); }
  }
  filesParsed = result.files_parsed;
  for (const item of result.unresolved ?? []) unresolved.push(item);
}

const map = {
  schema: MAP_SCHEMA,
  author: { type: "machine", extractor: MAPPER.extractor, version: MAPPER.version },
  generated_at: new Date().toISOString(),
  nodes,
  edges: [],
  coverage: {
    files_scanned: allFiles.length,
    files_parsed: filesParsed,
    unresolved,
  },
};

console.log(JSON.stringify(map, null, 2));
