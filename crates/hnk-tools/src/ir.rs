use std::collections::BTreeMap;
use std::path::PathBuf;

use hnk_idl::ast::{Annotation, ContractBag, OpaqueValue};
use hnk_idl::schema::{ConnectionLocality, PortDirection, Visibility};

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedSpec {
    pub version: String,
    pub entry_package: String,
    pub events: BTreeMap<String, NormalizedEvent>,
    pub components: BTreeMap<String, NormalizedComponent>,
    pub connections: Vec<NormalizedConnection>,
    pub non_local_connections: Vec<NormalizedConnection>,
    pub imports: Vec<String>,
    pub import_graph: BTreeMap<PathBuf, Vec<PathBuf>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedEvent {
    pub package: String,
    pub name: String,
    pub qualified_name: String,
    pub source_path: PathBuf,
    pub fields: Vec<NormalizedField>,
    pub visibility: Visibility,
    pub version: Option<String>,
    pub deprecated_since: Option<String>,
    pub deprecated_note: Option<String>,
    pub doc: Option<String>,
    pub kind: Option<String>,
    pub contracts: ContractBag,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedField {
    pub name: String,
    pub field_type: String,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedComponent {
    pub package: String,
    pub name: String,
    pub qualified_name: String,
    pub source_path: PathBuf,
    pub uses: BTreeMap<String, NormalizedComponentUse>,
    pub ports: BTreeMap<String, NormalizedPort>,
    pub state_fields: BTreeMap<String, NormalizedStateField>,
    pub transitions: BTreeMap<String, NormalizedTransition>,
    pub persistence: Option<NormalizedPersistence>,
    pub assumptions: Vec<Annotation>,
    pub doc: Option<String>,
    pub inbound_events: Vec<String>,
    pub outbound_events: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedPort {
    pub component_name: String,
    pub name: String,
    pub qualified_name: String,
    pub direction: PortDirection,
    pub event: String,
    pub contracts: ContractBag,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedComponentUse {
    pub name: String,
    pub component: String,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedStateField {
    pub component_name: String,
    pub name: String,
    pub field_type: String,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedTransition {
    pub component_name: String,
    pub name: String,
    pub on: String,
    pub emits: Vec<String>,
    pub reads: Vec<String>,
    pub writes: Vec<String>,
    pub requires: Vec<OpaqueValue>,
    pub ensures: Vec<OpaqueValue>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedPersistence {
    pub durable: Vec<String>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedConnection {
    pub within_component: String,
    pub from_target: String,
    pub from_component: String,
    pub from_port: String,
    pub to_target: String,
    pub to_component: String,
    pub to_port: String,
    pub locality: ConnectionLocality,
    pub contracts: ContractBag,
    pub doc: Option<String>,
}
