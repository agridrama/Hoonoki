use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::schema::{ConnectionLocality, PortDirection, Visibility};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub version: String,
    pub package: String,
    pub events: Vec<Event>,
    pub components: Vec<Component>,
    pub connections: Vec<Connection>,
    #[serde(default)]
    pub types: BTreeMap<String, TypeRef>,
    #[serde(default)]
    pub contracts: ContractBag,
    #[serde(default)]
    pub assumptions: Vec<Annotation>,
    #[serde(default)]
    pub imports: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub name: String,
    #[serde(default)]
    pub fields: Vec<Field>,
    pub visibility: Visibility,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub deprecated: Option<Deprecation>,
    #[serde(default)]
    pub doc: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub contracts: ContractBag,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: TypeRef,
    #[serde(default)]
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub name: String,
    pub ports: Vec<Port>,
    pub state: State,
    pub transitions: Vec<Transition>,
    #[serde(default)]
    pub uses: Vec<ComponentUse>,
    #[serde(default)]
    pub persistence: Option<Persistence>,
    #[serde(default)]
    pub assumptions: Vec<Annotation>,
    #[serde(default)]
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentUse {
    pub name: String,
    pub component: String,
    #[serde(default)]
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Port {
    pub name: String,
    pub direction: PortDirection,
    pub event: String,
    #[serde(default)]
    pub contracts: ContractBag,
    #[serde(default)]
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    #[serde(default)]
    pub fields: Vec<StateField>,
    #[serde(default)]
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: TypeRef,
    #[serde(default)]
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transition {
    pub name: String,
    pub on: String,
    #[serde(default)]
    pub emits: Vec<String>,
    #[serde(default)]
    pub reads: Vec<String>,
    #[serde(default)]
    pub writes: Vec<String>,
    #[serde(default)]
    pub requires: Vec<Annotation>,
    #[serde(default)]
    pub ensures: Vec<Annotation>,
    #[serde(default)]
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Connection {
    pub within: String,
    pub from: String,
    pub to: String,
    pub locality: ConnectionLocality,
    #[serde(default)]
    pub contracts: ContractBag,
    #[serde(default)]
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Persistence {
    #[serde(default)]
    pub durable: Vec<String>,
    #[serde(default)]
    pub doc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Deprecation {
    #[serde(default)]
    pub since: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

pub type TypeRef = String;
pub type ContractBag = BTreeMap<String, OpaqueValue>;
pub type Annotation = OpaqueValue;
pub type OpaqueValue = serde_yaml::Value;
