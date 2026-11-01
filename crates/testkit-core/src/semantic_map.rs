//! The semantic map contract: two witnesses of the same codebase.
//!
//! A [`SemanticMap`] lists the surfaces of an application (routes, handlers,
//! models, screens, commands) and how they connect. Two independent maps are
//! produced: the **machine map** (a framework-specific extractor, fast and
//! literal) and the **agent map** (the operator agent's own tour, attentive
//! but expensive). [`reconcile`] compares them: agreement is proof, and every
//! disagreement is a finding fed back to whichever witness missed it.
//!
//! Honesty rules baked into the contract:
//! - every map carries its author (extractor name+version, or `agent`);
//! - every map carries a coverage report listing what it could not read;
//! - a map that silently misses surfaces violates the contract.

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Current map schema version.
pub const MAP_VERSION: &str = "testkit.map/1";

/// Who drew this map. Provenance is part of the contract: receipts record it,
/// and the two witnesses must stay attributable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MapAuthor {
    /// A deterministic framework extractor.
    Machine {
        /// Extractor name, e.g. `extractor-next`.
        extractor: String,
        /// Extractor version string.
        version: String,
    },
    /// The operator agent's own tour of the codebase.
    Agent,
}

/// One surface of the application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapNode {
    /// Stable id, unique within the map.
    pub id: String,
    /// Surface kind: `route`, `handler`, `model`, `screen`, `command`,
    /// `button`, or any project-specific kind.
    pub kind: String,
    /// Human-readable name, e.g. `GET /api/trial`.
    pub name: String,
    /// Project-relative file path, when known.
    #[serde(default)]
    pub path: Option<String>,
    /// Free-form detail: HTTP method, button label, command synopsis.
    #[serde(default)]
    pub detail: String,
}

/// A connection between two surfaces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapEdge {
    /// Id of the source node.
    pub from: String,
    /// Id of the target node.
    pub to: String,
    /// Relationship: `calls`, `covers`, `proves`, `connects`, ...
    #[serde(default = "default_edge_kind")]
    pub kind: String,
}

fn default_edge_kind() -> String {
    "connects".to_string()
}

/// What a map could and could not read. An extractor that silently skips
/// files violates the contract; this report is the honesty seal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapCoverage {
    pub files_scanned: usize,
    pub files_parsed: usize,
    #[serde(default)]
    pub unresolved: Vec<UnresolvedPath>,
}

/// A path the map's author could not interpret, with the reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnresolvedPath {
    pub path: String,
    pub reason: String,
}

/// A semantic map: one witness's view of the application.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticMap {
    pub schema: String,
    pub author: MapAuthor,
    pub generated_at: DateTime<Utc>,
    pub nodes: Vec<MapNode>,
    #[serde(default)]
    pub edges: Vec<MapEdge>,
    pub coverage: MapCoverage,
}

impl SemanticMap {
    /// Parse and validate a map from JSON text.
    pub fn from_json(text: &str) -> Result<Self> {
        let map: Self =
            serde_json::from_str(text).context("parsing the semantic map JSON")?;
        map.validate()?;
        Ok(map)
    }

    /// Load, parse and validate a map file.
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?;
        Self::from_json(&text)
            .with_context(|| format!("validating map at {}", path.display()))
    }

    /// Enforce the contract: schema version, unique node ids, edges that
    /// reference declared nodes.
    pub fn validate(&self) -> Result<()> {
        if self.schema != MAP_VERSION {
            bail!(
                "map schema is '{}', expected '{MAP_VERSION}'",
                self.schema
            );
        }
        let mut ids = BTreeSet::new();
        for node in &self.nodes {
            if node.id.is_empty() {
                bail!("a node has an empty id");
            }
            if !ids.insert(node.id.as_str()) {
                bail!("duplicate node id '{}'", node.id);
            }
        }
        for edge in &self.edges {
            if !ids.contains(edge.from.as_str()) {
                bail!("edge references unknown source node '{}'", edge.from);
            }
            if !ids.contains(edge.to.as_str()) {
                bail!("edge references unknown target node '{}'", edge.to);
            }
        }
        Ok(())
    }

    /// The comparison key for reconciliation: kind plus location. Two
    /// witnesses describing the same door carry the same key even if they
    /// disagree about the door's name.
    pub fn node_key(node: &MapNode) -> String {
        let location = node
            .path
            .as_deref()
            .filter(|path| !path.is_empty())
            .unwrap_or(node.name.as_str());
        format!("{}:{location}", node.kind)
    }
}

/// The collaboration output: where the two witnesses agree and differ.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reconciliation {
    pub schema: String,
    pub generated_at: DateTime<Utc>,
    pub machine_author: MapAuthor,
    pub agent_author: MapAuthor,
    /// Node keys present in both maps: agreement is proof.
    pub agreed: usize,
    /// Nodes only the machine map found — the agent's tour missed them.
    pub machine_only: Vec<MapNode>,
    /// Nodes only the agent map found — the extractor's blind spots.
    pub agent_only: Vec<MapNode>,
    /// What the machine map could not read.
    pub machine_unresolved: Vec<UnresolvedPath>,
    /// What the agent tour could not read.
    pub agent_unresolved: Vec<UnresolvedPath>,
}

impl Reconciliation {
    pub fn schema_version() -> &'static str {
        "testkit.reconciliation/1"
    }
}

/// Compare the two witnesses. Machine-only nodes accuse the agent's tour;
/// agent-only nodes accuse the extractor. Both lists are findings, not
/// failures — they are the collaboration's most valuable output.
pub fn reconcile(machine: &SemanticMap, agent: &SemanticMap) -> Reconciliation {
    let machine_keys: BTreeSet<String> =
        machine.nodes.iter().map(SemanticMap::node_key).collect();
    let agent_keys: BTreeSet<String> = agent.nodes.iter().map(SemanticMap::node_key).collect();

    let mut agreed = 0usize;
    let mut machine_only = Vec::new();
    for node in &machine.nodes {
        if agent_keys.contains(&SemanticMap::node_key(node)) {
            agreed += 1;
        } else {
            machine_only.push(node.clone());
        }
    }
    let mut agent_only = Vec::new();
    for node in &agent.nodes {
        if !machine_keys.contains(&SemanticMap::node_key(node)) {
            agent_only.push(node.clone());
        }
    }

    Reconciliation {
        schema: Reconciliation::schema_version().to_string(),
        generated_at: Utc::now(),
        machine_author: machine.author.clone(),
        agent_author: agent.author.clone(),
        agreed,
        machine_only,
        agent_only,
        machine_unresolved: machine.coverage.unresolved.clone(),
        agent_unresolved: agent.coverage.unresolved.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str, kind: &str, path: &str) -> MapNode {
        MapNode {
            id: id.to_string(),
            kind: kind.to_string(),
            name: id.to_string(),
            path: Some(path.to_string()),
            detail: String::new(),
        }
    }

    fn sample_map(author: MapAuthor, nodes: Vec<MapNode>) -> SemanticMap {
        SemanticMap {
            schema: MAP_VERSION.to_string(),
            author,
            generated_at: Utc::now(),
            nodes,
            edges: Vec::new(),
            coverage: MapCoverage {
                files_scanned: 10,
                files_parsed: 9,
                unresolved: vec![UnresolvedPath {
                    path: "src/legacy.js".to_string(),
                    reason: "unrecognized routing style".to_string(),
                }],
            },
        }
    }

    #[test]
    fn validate_accepts_well_formed_maps() {
        let map = sample_map(
            MapAuthor::Agent,
            vec![node("a", "route", "src/a.ts"), node("b", "model", "src/b.ts")],
        );
        assert!(map.validate().is_ok());
    }

    #[test]
    fn validate_rejects_duplicate_ids_and_dangling_edges() {
        let mut map = sample_map(
            MapAuthor::Agent,
            vec![node("a", "route", "src/a.ts"), node("a", "model", "src/b.ts")],
        );
        assert!(map.validate().is_err());

        map.nodes = vec![node("a", "route", "src/a.ts")];
        map.edges = vec![MapEdge {
            from: "a".to_string(),
            to: "ghost".to_string(),
            kind: "calls".to_string(),
        }];
        assert!(map.validate().is_err());
    }

    #[test]
    fn validate_rejects_wrong_schema() {
        let mut map = sample_map(MapAuthor::Agent, vec![node("a", "route", "src/a.ts")]);
        map.schema = "testkit.map/0".to_string();
        assert!(map.validate().is_err());
    }

    #[test]
    fn reconcile_reports_agreement_and_both_blind_spots() {
        let machine = sample_map(
            MapAuthor::Machine {
                extractor: "extractor-next".to_string(),
                version: "0.1.0".to_string(),
            },
            vec![
                node("r1", "route", "src/app/api/trial/route.ts"),
                node("r2", "route", "src/app/api/hidden/route.ts"),
            ],
        );
        let agent = sample_map(
            MapAuthor::Agent,
            vec![
                node("a1", "route", "src/app/api/trial/route.ts"),
                node("a2", "route", "src/handmade/checkout.ts"),
            ],
        );

        let report = reconcile(&machine, &agent);
        assert_eq!(report.agreed, 1);
        assert_eq!(report.machine_only.len(), 1);
        assert_eq!(report.machine_only[0].id, "r2");
        assert_eq!(report.agent_only.len(), 1);
        assert_eq!(report.agent_only[0].id, "a2");
        assert_eq!(report.machine_unresolved.len(), 1);
        assert_eq!(report.agent_unresolved.len(), 1);
    }

    #[test]
    fn node_key_matches_on_kind_and_path_despite_different_names() {
        let machine_node = MapNode {
            id: "m1".to_string(),
            kind: "route".to_string(),
            name: "GET /api/trial".to_string(),
            path: Some("src/app/api/trial/route.ts".to_string()),
            detail: String::new(),
        };
        let agent_node = MapNode {
            id: "a1".to_string(),
            kind: "route".to_string(),
            name: "trial endpoint".to_string(),
            path: Some("src/app/api/trial/route.ts".to_string()),
            detail: String::new(),
        };
        assert_eq!(
            SemanticMap::node_key(&machine_node),
            SemanticMap::node_key(&agent_node)
        );
    }
}
