//! `testkit map` — orchestrate the two-witness semantic map.
//!
//! The flow, fully deterministic on the engine side:
//! 1. **Detect** the stack from dependency manifests (never the agent's
//!    guess) and record it in `.testkit/brain/stack.json`.
//! 2. **Machine map**: run `extractor-<framework>` from
//!    `.testkit/extractors/` or `~/.testkit/extractors/` when one exists.
//!    When none exists, the agent is the mapper — TestKit works with any
//!    framework from day one.
//! 3. **Agent map**: when `.testkit/brain/map.agent.json` is present it is
//!    validated; otherwise `testkit map --agent-brief` prints the standing
//!    tour instructions.
//! 4. **Reconcile**: compare both witnesses and write
//!    `.testkit/brain/map.reconciliation.json`. Agreement is proof;
//!    disagreement is a finding for whichever witness missed it.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use serde::Serialize;

use testkit_core::semantic_map::{Reconciliation, SemanticMap, MAP_VERSION};

/// The detected stack, recorded for audit: the engine's guess, with the
/// evidence of what it guessed from.
#[derive(Debug, Clone, Serialize)]
struct StackProfile {
    schema: String,
    language: String,
    framework: Option<String>,
    detected_from: Vec<String>,
}

/// Run the map orchestration. `agent_brief` prints only the standing tour
/// instructions for the operator agent.
pub fn run(agent_brief: bool) -> Result<()> {
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let brain = root.join(".testkit").join("brain");
    if !root.join(".testkit").is_dir() || !brain.is_dir() {
        eprintln!("TestKit not initialized here. Run `testkit init` first.");
        std::process::exit(2);
    }

    if agent_brief {
        print_agent_brief(&root);
        return Ok(());
    }

    // 1. Detect the stack deterministically.
    let stack = detect_stack(&root);
    let stack_path = brain.join("stack.json");
    fs::write(
        &stack_path,
        serde_json::to_vec_pretty(&stack).context("serializing the stack profile")?,
    )
    .with_context(|| format!("writing {}", stack_path.display()))?;
    println!(
        "Stack: {} ({})",
        stack.framework.as_deref().unwrap_or("unrecognized"),
        stack.detected_from.join(", ")
    );

    // 2. Machine map: dedicated extractor binaries first, then the bundled
    //    grammar-driven mapper, then the agent.
    let mut machine_map = run_machine_mapper(&root, &stack)?;
    if machine_map.is_none() {
        machine_map = run_generic_mapper(&root);
    }
    match &machine_map {
        Some(map) => {
            let path = brain.join("map.machine.json");
            fs::write(&path, serde_json::to_vec_pretty(map).context("serializing machine map")?)
                .with_context(|| format!("writing {}", path.display()))?;
            println!(
                "Machine map: {} nodes (extractor {}), {} unresolved",
                map.nodes.len(),
                match &map.author {
                    testkit_core::MapAuthor::Machine { extractor, .. } => extractor.as_str(),
                    _ => "unknown",
                },
                map.coverage.unresolved.len(),
            );
        }
        None => {
            println!(
                "No machine map was produced. Two ways to fix that:\n\
                 1. Grammar-driven (preferred): the agent installs the Tree-sitter grammar\n\
                    once for this project — `npm install --save-dev web-tree-sitter tree-sitter-wasms`\n\
                    — then re-runs `testkit map` to drive .testkit/extractors/generic-mapper.mjs.\n\
                 2. Agent-authored: run `testkit map --agent-brief`, follow the tour, and\n\
                    write .testkit/brain/map.agent.json."
            );
        }
    }

    // 3. Agent map, when the tour has been performed.
    let agent_path = brain.join("map.agent.json");
    if !agent_path.exists() {
        println!(
            "No agent map yet at {}. Run `testkit map --agent-brief`, perform the tour, \
             then re-run `testkit map` to reconcile.",
            agent_path.display()
        );
        return Ok(());
    }
    let agent_map = SemanticMap::load(&agent_path)
        .context("the agent map failed validation; fix it and re-run `testkit map`")?;

    // 4. Reconcile whatever witnesses exist.
    let reconciliation = match &machine_map {
        Some(machine) => reconcile_and_write(&brain, machine, &agent_map)?,
        None => {
            println!("Agent map: {} nodes (single witness; no machine map to reconcile against). Run again once a machine mapper exists.", agent_map.nodes.len());
            return Ok(());
        }
    };

    println!(
        "Reconciliation: {} agreed, {} machine-only, {} agent-only",
        reconciliation.agreed,
        reconciliation.machine_only.len(),
        reconciliation.agent_only.len()
    );
    if !reconciliation.agent_only.is_empty() {
        println!(
            "The agent found doors the extractor missed — candidates for the next \
             extractor fix or new invariants."
        );
    }
    if !reconciliation.machine_only.is_empty() {
        println!(
            "The extractor found doors the agent's tour missed — the tour should \
             revisit them."
        );
    }
    Ok(())
}

/// Attempt the machine extractor for the detected framework. Returns `None`
/// when no extractor is installed — never an error: the agent fallback keeps
/// every framework supported.
fn run_machine_mapper(
    root: &Path,
    stack: &StackProfile,
) -> Result<Option<SemanticMap>> {
    let framework = match &stack.framework {
        Some(framework) => framework.clone(),
        None => return Ok(None),
    };

    let mut search_dirs = vec![root.join(".testkit").join("extractors")];
    if let Some(home) = dirs::home_dir() {
        search_dirs.push(home.join(".testkit").join("extractors"));
    }

    for dir in &search_dirs {
        for candidate in [format!("extractor-{framework}"), format!("extractor-{framework}.exe")] {
            let extractor = dir.join(&candidate);
            if !extractor.is_file() {
                continue;
            }
            let output = Command::new(extractor)
                .arg(root)
                .output()
                .with_context(|| format!("running extractor {candidate}"))?;
            if !output.status.success() {
                anyhow::bail!(
                    "extractor {candidate} failed (exit {:?}): {}",
                    output.status.code(),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            let map = SemanticMap::from_json(&String::from_utf8_lossy(&output.stdout))
                .context("the extractor emitted an invalid semantic map")?;
            return Ok(Some(map));
        }
    }
    Ok(None)
}

/// Try the bundled grammar-driven mapper (`testkit init` drops it into
/// `.testkit/extractors/`). TestKit builds no per-framework parsers: the
/// community's Tree-sitter grammars do the parsing (the agent installs
/// `web-tree-sitter` + `tree-sitter-wasms` once), this script applies the
/// per-framework convention rules and emits the map. Any failure here is
/// advisory — the agent fallback keeps mapping available for every stack.
fn run_generic_mapper(root: &Path) -> Option<SemanticMap> {
    let script = root
        .join(".testkit")
        .join("extractors")
        .join("generic-mapper.mjs");
    if !script.is_file() {
        return None;
    }
    let output = match Command::new("node").arg(&script).arg("--root").arg(root).output() {
        Ok(output) => output,
        Err(error) => {
            eprintln!("Generic mapper not run (is node on PATH?): {error}");
            return None;
        }
    };
    if !output.status.success() {
        eprintln!(
            "Generic mapper could not map this stack:\n{}",
            String::from_utf8_lossy(&output.stderr).trim_end()
        );
        return None;
    }
    match SemanticMap::from_json(&String::from_utf8_lossy(&output.stdout)) {
        Ok(map) => Some(map),
        Err(error) => {
            eprintln!("Generic mapper emitted an invalid map: {error}");
            None
        }
    }
}

/// Validate both maps and write the reconciliation artifact.
fn reconcile_and_write(
    brain: &Path,
    machine: &SemanticMap,
    agent: &SemanticMap,
) -> Result<Reconciliation> {
    let report = testkit_core::semantic_map::reconcile(machine, agent);
    let path = brain.join("map.reconciliation.json");
    fs::write(
        &path,
        serde_json::to_vec_pretty(&report).context("serializing the reconciliation")?,
    )
    .with_context(|| format!("writing {}", path.display()))?;
    Ok(report)
}

/// Print the standing tour instructions for the operator agent.
fn print_agent_brief(root: &Path) {
    println!(
        r#"AGENT TOUR BRIEF — write .testkit/brain/map.agent.json

You are the second witness of the codebase at {root}. A machine map may
already exist at .testkit/brain/map.machine.json — do NOT read it, do NOT
correct it, and do NOT copy from it. Draw your own floor plan first;
reconciliation compares the two.

Walk the codebase yourself and list every surface the application exposes,
translated into whatever this platform calls them:
  - web/API projects: every HTTP route and handler, every form/button and
    the API it calls, every database model
  - mobile projects: every screen, intent, deep link, and local store
  - CLIs: every command, flag, and exit code contract

Schema: {MAP_VERSION} (see testkit-core/src/semantic_map.rs for the full
contract). Required honesty rules:
  - author: {{"type": "agent"}}
  - nodes: [{{ "id", "kind", "name", "path", "detail" }}] — path is
    project-relative and must be exact; ids unique
  - edges: [{{ "from", "to", "kind" }}] referencing declared ids
  - coverage: files_scanned, files_parsed, unresolved: [{{ path, reason }}]
    — every file you could not interpret goes in `unresolved`, with why.

When done, re-run `testkit map` to validate your map and reconcile it
against the machine map (if one exists)."#
    );
}

// ---------------------------------------------------------------------------
// Stack detection — file-based, deterministic, never the agent's guess
// ---------------------------------------------------------------------------

/// Ordered candidate signatures; more specific frameworks first so a Next.js
/// monorepo that also uses Express reports Next.js.
fn detect_stack(root: &Path) -> StackProfile {
    let mut detected_from = Vec::new();

    if let Some(profile) = detect_from_package_json(root, &mut detected_from) {
        return profile;
    }
    if let Some(profile) = detect_from_python(root, &mut detected_from) {
        return profile;
    }
    if let Some(profile) = detect_from_cargo(root, &mut detected_from) {
        return profile;
    }
    if let Some(profile) = detect_from_go(root, &mut detected_from) {
        return profile;
    }
    if let Some(profile) = detect_from_gradle(root, &mut detected_from) {
        return profile;
    }
    if has_extension_dir(root, "xcodeproj") {
        detected_from.push("*.xcodeproj".to_string());
        return StackProfile {
            schema: "testkit.stack/1".to_string(),
            language: "swift".to_string(),
            framework: Some("ios".to_string()),
            detected_from,
        };
    }
    StackProfile {
        schema: "testkit.stack/1".to_string(),
        language: "unknown".to_string(),
        framework: None,
        detected_from: vec!["no recognized manifest".to_string()],
    }
}

fn read_manifest(root: &Path, name: &str) -> Option<String> {
    fs::read_to_string(root.join(name)).ok()
}

/// A dependency present in a parsed package.json `dependencies` or
/// `devDependencies` block.
fn has_dependency(manifest: &str, package: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(manifest)
        .ok()
        .map(|value| {
            ["dependencies", "devDependencies", "peerDependencies"]
                .iter()
                .any(|section| {
                    value
                        .get(section)
                        .and_then(|deps| deps.get(package))
                        .is_some()
                })
        })
        .unwrap_or(false)
}

fn detect_from_package_json(root: &Path, detected_from: &mut Vec<String>) -> Option<StackProfile> {
    let manifest = read_manifest(root, "package.json")?;
    detected_from.push("package.json".to_string());
    const JS_SIGS: [(&str, &str, &str); 9] = [
        ("@nestjs/core", "nestjs", "typescript"),
        ("next", "nextjs", "typescript"),
        ("nuxt", "nuxt", "typescript"),
        ("@angular/core", "angular", "typescript"),
        ("@sveltejs/kit", "sveltekit", "typescript"),
        ("vue", "vue", "typescript"),
        ("react", "react", "typescript"),
        ("express", "express", "javascript"),
        ("fastify", "fastify", "javascript"),
    ];
    for (package, framework, language) in JS_SIGS {
        if has_dependency(&manifest, package) {
            detected_from.push(format!("{package} declared"));
            return Some(StackProfile {
                schema: "testkit.stack/1".to_string(),
                language: language.to_string(),
                framework: Some(framework.to_string()),
                detected_from: detected_from.clone(),
            });
        }
    }
    Some(StackProfile {
        schema: "testkit.stack/1".to_string(),
        language: "javascript".to_string(),
        framework: None,
        detected_from: detected_from.clone(),
    })
}

fn detect_from_python(root: &Path, detected_from: &mut Vec<String>) -> Option<StackProfile> {
    let manifest = read_manifest(root, "pyproject.toml")
        .or_else(|| read_manifest(root, "requirements.txt"))?;
    detected_from.push("python manifest".to_string());
    const PY_SIGS: [(&str, &str); 3] = [
        ("fastapi", "fastapi"),
        ("django", "django"),
        ("flask", "flask"),
    ];
    for (needle, framework) in PY_SIGS {
        if manifest.contains(needle) {
            detected_from.push(format!("{needle} declared"));
            return Some(StackProfile {
                schema: "testkit.stack/1".to_string(),
                language: "python".to_string(),
                framework: Some(framework.to_string()),
                detected_from: detected_from.clone(),
            });
        }
    }
    Some(StackProfile {
        schema: "testkit.stack/1".to_string(),
        language: "python".to_string(),
        framework: None,
        detected_from: detected_from.clone(),
    })
}

fn detect_from_cargo(root: &Path, detected_from: &mut Vec<String>) -> Option<StackProfile> {
    let manifest = read_manifest(root, "Cargo.toml")?;
    detected_from.push("Cargo.toml".to_string());
    const RUST_SIGS: [(&str, &str); 3] = [
        ("actix-web", "actix"),
        ("axum", "axum"),
        ("rocket", "rocket"),
    ];
    for (needle, framework) in RUST_SIGS {
        if manifest.contains(needle) {
            detected_from.push(format!("{needle} declared"));
            return Some(StackProfile {
                schema: "testkit.stack/1".to_string(),
                language: "rust".to_string(),
                framework: Some(framework.to_string()),
                detected_from: detected_from.clone(),
            });
        }
    }
    Some(StackProfile {
        schema: "testkit.stack/1".to_string(),
        language: "rust".to_string(),
        framework: None,
        detected_from: detected_from.clone(),
    })
}

fn detect_from_go(root: &Path, detected_from: &mut Vec<String>) -> Option<StackProfile> {
    let manifest = read_manifest(root, "go.mod")?;
    detected_from.push("go.mod".to_string());
    const GO_SIGS: [(&str, &str); 3] = [
        ("gin-gonic", "gin"),
        ("labstack/echo", "echo"),
        ("go-chi", "chi"),
    ];
    for (needle, framework) in GO_SIGS {
        if manifest.contains(needle) {
            detected_from.push(format!("{needle} declared"));
            return Some(StackProfile {
                schema: "testkit.stack/1".to_string(),
                language: "go".to_string(),
                framework: Some(framework.to_string()),
                detected_from: detected_from.clone(),
            });
        }
    }
    Some(StackProfile {
        schema: "testkit.stack/1".to_string(),
        language: "go".to_string(),
        framework: None,
        detected_from: detected_from.clone(),
    })
}

fn detect_from_gradle(root: &Path, detected_from: &mut Vec<String>) -> Option<StackProfile> {
    let manifest = read_manifest(root, "build.gradle")
        .or_else(|| read_manifest(root, "build.gradle.kts"))?;
    detected_from.push("gradle build script".to_string());
    if manifest.contains("com.android") {
        detected_from.push("com.android plugin".to_string());
        return Some(StackProfile {
            schema: "testkit.stack/1".to_string(),
            language: "kotlin/java".to_string(),
            framework: Some("android".to_string()),
            detected_from: detected_from.clone(),
        });
    }
    if manifest.contains("spring-boot") || manifest.contains("org.springframework.boot") {
        detected_from.push("spring-boot declared".to_string());
        return Some(StackProfile {
            schema: "testkit.stack/1".to_string(),
            language: "java/kotlin".to_string(),
            framework: Some("spring".to_string()),
            detected_from: detected_from.clone(),
        });
    }
    Some(StackProfile {
        schema: "testkit.stack/1".to_string(),
        language: "java/kotlin".to_string(),
        framework: None,
        detected_from: detected_from.clone(),
    })
}

fn has_extension_dir(root: &Path, extension: &str) -> bool {
    fs::read_dir(root)
        .map(|entries| {
            entries
                .flatten()
                .any(|entry| {
                    entry
                        .path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext == extension)
                        .unwrap_or(false)
                })
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_project(label: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "testkit-map-{label}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn detects_nextjs_from_package_json() {
        let root = temp_project("next");
        fs::write(
            root.join("package.json"),
            r#"{ "dependencies": { "next": "15.1.0", "react": "19.0.0" } }"#,
        )
        .unwrap();
        let stack = detect_stack(&root);
        assert_eq!(stack.framework.as_deref(), Some("nextjs"));
        assert!(stack.detected_from.iter().any(|line| line.contains("next declared")));
    }

    #[test]
    fn prefers_the_most_specific_framework_signature() {
        let root = temp_project("nest");
        fs::write(
            root.join("package.json"),
            r#"{ "dependencies": { "@nestjs/core": "10.0.0", "express": "4.19.0" } }"#,
        )
        .unwrap();
        let stack = detect_stack(&root);
        assert_eq!(stack.framework.as_deref(), Some("nestjs"));
    }

    #[test]
    fn detects_fastapi_from_python_manifest() {
        let root = temp_project("fastapi");
        fs::write(
            root.join("requirements.txt"),
            "fastapi==0.115.0\nuvicorn==0.30.0\n",
        )
        .unwrap();
        let stack = detect_stack(&root);
        assert_eq!(stack.framework.as_deref(), Some("fastapi"));
        assert_eq!(stack.language, "python");
    }

    #[test]
    fn unknown_stack_reports_no_framework_without_failing() {
        let root = temp_project("empty");
        let stack = detect_stack(&root);
        assert_eq!(stack.framework, None);
        assert_eq!(stack.language, "unknown");
    }
}
