use std::collections::{BTreeMap, BTreeSet, HashMap};

use hnk_idl::Spec;
use hnk_idl::diagnostics::{Diagnostic, DiagnosticSet};
use hnk_idl::schema::PortDirection;

use crate::ir::{
    NormalizedActor, NormalizedComponent, NormalizedConnection, NormalizedEvent, NormalizedField,
    NormalizedPersistence, NormalizedPort, NormalizedSpec, NormalizedStateField,
    NormalizedTransition,
};

pub fn normalize(spec: &Spec) -> Result<NormalizedSpec, DiagnosticSet> {
    let mut diagnostics = Vec::new();

    let event_index = collect_event_index(spec, &mut diagnostics);
    let actor_index = collect_actor_index(spec, &mut diagnostics);
    let components = collect_components(spec, &event_index, &mut diagnostics);
    let connections = collect_connections(spec, &components, &actor_index, &mut diagnostics);

    if !diagnostics.is_empty() {
        return Err(DiagnosticSet::new(diagnostics));
    }

    let actors = spec
        .actors
        .iter()
        .map(|actor| {
            (
                actor.name.clone(),
                NormalizedActor {
                    name: actor.name.clone(),
                    components: actor.components.clone(),
                    role_selector: actor.role_selector.clone(),
                    routing: actor.routing.clone(),
                    doc: actor.doc.clone(),
                },
            )
        })
        .collect();

    let actor_boundary_crossings = connections
        .iter()
        .filter(|connection| connection.crosses_actor_boundary)
        .cloned()
        .collect();

    Ok(NormalizedSpec {
        version: spec.version.clone(),
        events: event_index,
        components,
        connections,
        actors,
        actor_boundary_crossings,
        imports: spec.imports.clone(),
    })
}

fn collect_event_index(
    spec: &Spec,
    diagnostics: &mut Vec<Diagnostic>,
) -> BTreeMap<String, NormalizedEvent> {
    let mut events = BTreeMap::new();

    for event in &spec.events {
        if events.contains_key(&event.name) {
            diagnostics.push(error(
                "HNK2001",
                format!("Duplicate event name `{}`.", event.name),
            ));
            continue;
        }

        events.insert(
            event.name.clone(),
            NormalizedEvent {
                name: event.name.clone(),
                fields: event
                    .fields
                    .iter()
                    .map(|field| NormalizedField {
                        name: field.name.clone(),
                        field_type: field.field_type.clone(),
                        doc: field.doc.clone(),
                    })
                    .collect(),
                visibility: event.visibility.clone(),
                version: event.version.clone(),
                deprecated_since: event.deprecated.as_ref().and_then(|it| it.since.clone()),
                deprecated_note: event.deprecated.as_ref().and_then(|it| it.note.clone()),
                doc: event.doc.clone(),
                kind: event.kind.clone(),
                contracts: event.contracts.clone(),
            },
        );
    }

    events
}

fn collect_actor_index(
    spec: &Spec,
    diagnostics: &mut Vec<Diagnostic>,
) -> HashMap<String, String> {
    let mut actor_index = HashMap::new();

    for actor in &spec.actors {
        for component in &actor.components {
            if let Some(existing) = actor_index.insert(component.clone(), actor.name.clone()) {
                diagnostics.push(error(
                    "HNK2002",
                    format!(
                        "Component `{component}` is assigned to multiple actors: `{existing}` and `{}`.",
                        actor.name
                    ),
                ));
            }
        }
    }

    actor_index
}

fn collect_components(
    spec: &Spec,
    events: &BTreeMap<String, NormalizedEvent>,
    diagnostics: &mut Vec<Diagnostic>,
) -> BTreeMap<String, NormalizedComponent> {
    let mut components = BTreeMap::new();

    for component in &spec.components {
        if components.contains_key(&component.name) {
            diagnostics.push(error(
                "HNK2003",
                format!("Duplicate component name `{}`.", component.name),
            ));
            continue;
        }

        let mut ports = BTreeMap::new();
        let mut inbound_events = BTreeSet::new();
        let mut outbound_events = BTreeSet::new();

        for port in &component.ports {
            if ports.contains_key(&port.name) {
                diagnostics.push(error(
                    "HNK2004",
                    format!(
                        "Component `{}` defines port `{}` more than once.",
                        component.name, port.name
                    ),
                ));
                continue;
            }

            if !events.contains_key(&port.event) {
                diagnostics.push(error(
                    "HNK2005",
                    format!(
                        "Component `{}` port `{}` references undefined event `{}`.",
                        component.name, port.name, port.event
                    ),
                ));
            }

            match port.direction {
                PortDirection::In => {
                    inbound_events.insert(port.event.clone());
                }
                PortDirection::Out => {
                    outbound_events.insert(port.event.clone());
                }
            }

            ports.insert(
                port.name.clone(),
                NormalizedPort {
                    component_name: component.name.clone(),
                    name: port.name.clone(),
                    qualified_name: format!("{}.{}", component.name, port.name),
                    direction: port.direction.clone(),
                    event: port.event.clone(),
                    contracts: port.contracts.clone(),
                    doc: port.doc.clone(),
                },
            );
        }

        let mut state_fields = BTreeMap::new();
        for field in &component.state.fields {
            if state_fields.contains_key(&field.name) {
                diagnostics.push(error(
                    "HNK2006",
                    format!(
                        "Component `{}` defines state field `{}` more than once.",
                        component.name, field.name
                    ),
                ));
                continue;
            }

            state_fields.insert(
                field.name.clone(),
                NormalizedStateField {
                    component_name: component.name.clone(),
                    name: field.name.clone(),
                    field_type: field.field_type.clone(),
                    doc: field.doc.clone(),
                },
            );
        }

        let mut transitions = BTreeMap::new();
        for transition in &component.transitions {
            if transitions.contains_key(&transition.name) {
                diagnostics.push(error(
                    "HNK2007",
                    format!(
                        "Component `{}` defines transition `{}` more than once.",
                        component.name, transition.name
                    ),
                ));
                continue;
            }

            if !events.contains_key(&transition.on) {
                diagnostics.push(error(
                    "HNK2008",
                    format!(
                        "Transition `{}.{}` references undefined input event `{}`.",
                        component.name, transition.name, transition.on
                    ),
                ));
            }

            for event_name in &transition.emits {
                if !events.contains_key(event_name) {
                    diagnostics.push(error(
                        "HNK2009",
                        format!(
                            "Transition `{}.{}` emits undefined event `{event_name}`.",
                            component.name, transition.name
                        ),
                    ));
                }
            }

            transitions.insert(
                transition.name.clone(),
                NormalizedTransition {
                    component_name: component.name.clone(),
                    name: transition.name.clone(),
                    on: transition.on.clone(),
                    emits: transition.emits.clone(),
                    reads: transition.reads.clone(),
                    writes: transition.writes.clone(),
                    requires: transition.requires.clone(),
                    ensures: transition.ensures.clone(),
                    doc: transition.doc.clone(),
                },
            );
        }

        components.insert(
            component.name.clone(),
            NormalizedComponent {
                name: component.name.clone(),
                ports,
                state_fields,
                transitions,
                persistence: component.persistence.as_ref().map(|persistence| NormalizedPersistence {
                    durable: persistence.durable.clone(),
                    doc: persistence.doc.clone(),
                }),
                assumptions: component.assumptions.clone(),
                doc: component.doc.clone(),
                inbound_events: inbound_events.into_iter().collect(),
                outbound_events: outbound_events.into_iter().collect(),
            },
        );
    }

    components
}

fn collect_connections(
    spec: &Spec,
    components: &BTreeMap<String, NormalizedComponent>,
    actor_index: &HashMap<String, String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<NormalizedConnection> {
    let mut connections = Vec::new();

    for connection in &spec.connections {
        let Some((from_component, from_port)) = parse_endpoint(&connection.from) else {
            diagnostics.push(error(
                "HNK2010",
                format!(
                    "Connection `from` endpoint `{}` must use `component.port` format.",
                    connection.from
                ),
            ));
            continue;
        };
        let Some((to_component, to_port)) = parse_endpoint(&connection.to) else {
            diagnostics.push(error(
                "HNK2011",
                format!(
                    "Connection `to` endpoint `{}` must use `component.port` format.",
                    connection.to
                ),
            ));
            continue;
        };

        let Some(from_component_model) = components.get(from_component) else {
            diagnostics.push(error(
                "HNK2012",
                format!(
                    "Connection `from` references undefined component `{from_component}`."
                ),
            ));
            continue;
        };
        let Some(to_component_model) = components.get(to_component) else {
            diagnostics.push(error(
                "HNK2013",
                format!("Connection `to` references undefined component `{to_component}`."),
            ));
            continue;
        };
        let Some(_from_port_model) = from_component_model.ports.get(from_port) else {
            diagnostics.push(error(
                "HNK2014",
                format!(
                    "Connection `from` references undefined port `{}.{from_port}`.",
                    from_component
                ),
            ));
            continue;
        };
        let Some(_to_port_model) = to_component_model.ports.get(to_port) else {
            diagnostics.push(error(
                "HNK2015",
                format!(
                    "Connection `to` references undefined port `{}.{to_port}`.",
                    to_component
                ),
            ));
            continue;
        };

        let from_actor = actor_index.get(from_component).cloned();
        let to_actor = actor_index.get(to_component).cloned();
        let crosses_actor_boundary = from_actor != to_actor;

        connections.push(NormalizedConnection {
            from_component: from_component.to_string(),
            from_port: from_port.to_string(),
            to_component: to_component.to_string(),
            to_port: to_port.to_string(),
            contracts: connection.contracts.clone(),
            doc: connection.doc.clone(),
            from_actor,
            to_actor,
            crosses_actor_boundary,
        });
    }

    connections
}

fn parse_endpoint(endpoint: &str) -> Option<(&str, &str)> {
    let (component, port) = endpoint.split_once('.')?;
    if component.is_empty() || port.is_empty() {
        return None;
    }
    Some((component, port))
}

fn error(code: &'static str, message: String) -> Diagnostic {
    Diagnostic::new(hnk_idl::diagnostics::Severity::Error, code, message, None, None)
}
