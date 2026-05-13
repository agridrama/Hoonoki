use std::collections::BTreeMap;

use hnk_idl::ast::{Annotation, ContractBag, OpaqueValue};
use hnk_idl::schema::{PortDirection, Visibility};

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedSpec {
    pub version: String,
    pub events: BTreeMap<String, NormalizedEvent>,
    pub components: BTreeMap<String, NormalizedComponent>,
    pub connections: Vec<NormalizedConnection>,
    pub actors: BTreeMap<String, NormalizedActor>,
    pub actor_boundary_crossings: Vec<NormalizedConnection>,
    pub imports: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedEvent {
    pub name: String,
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
    pub name: String,
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
    pub from_component: String,
    pub from_port: String,
    pub to_component: String,
    pub to_port: String,
    pub contracts: ContractBag,
    pub doc: Option<String>,
    pub from_actor: Option<String>,
    pub to_actor: Option<String>,
    pub crosses_actor_boundary: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedActor {
    pub name: String,
    pub components: Vec<String>,
    pub role_selector: Option<String>,
    pub routing: Option<OpaqueValue>,
    pub doc: Option<String>,
}
