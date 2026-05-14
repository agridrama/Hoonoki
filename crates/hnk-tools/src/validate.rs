use std::collections::{BTreeSet, HashMap, VecDeque};

use hnk_idl::Spec;
use hnk_idl::diagnostics::{Diagnostic, DiagnosticSet, Severity};
use hnk_idl::schema::{PortDirection, Visibility};

use crate::contracts::has_must_reply;
use crate::ir::{NormalizedComponent, NormalizedConnection, NormalizedSpec};
use crate::normalize::normalize;

pub fn validate(spec: &Spec) -> DiagnosticSet {
    match normalize(spec) {
        Ok(normalized) => validate_normalized(&normalized),
        Err(diagnostics) => diagnostics,
    }
}

pub fn validate_normalized(spec: &NormalizedSpec) -> DiagnosticSet {
    let mut diagnostics = Vec::new();

    validate_event_versions(spec, &mut diagnostics);
    validate_connections(spec, &mut diagnostics);
    validate_transitions(spec, &mut diagnostics);
    validate_persistence(spec, &mut diagnostics);
    validate_must_reply(spec, &mut diagnostics);

    DiagnosticSet::new(diagnostics)
}

fn validate_event_versions(spec: &NormalizedSpec, diagnostics: &mut Vec<Diagnostic>) {
    for event in spec.events.values() {
        if matches!(event.visibility, Visibility::Public) && event.version.is_none() {
            diagnostics.push(error(
                    "HNK2101",
                format!(
                    "Public event `{}` must declare a `version` field.",
                    event.qualified_name
                ),
            ));
        }
    }
}

fn validate_connections(spec: &NormalizedSpec, diagnostics: &mut Vec<Diagnostic>) {
    for connection in &spec.connections {
        let from_component = &spec.components[&connection.from_component];
        let to_component = &spec.components[&connection.to_component];
        let from_port = &from_component.ports[&connection.from_port];
        let to_port = &to_component.ports[&connection.to_port];

        if !matches!(from_port.direction, PortDirection::Out) {
            diagnostics.push(error(
                "HNK2103",
                format!(
                    "Connection source `{}.{}` inside `{}` must be an `out` port.",
                    connection.from_target, connection.from_port, connection.within_component
                ),
            ));
        }
        if !matches!(to_port.direction, PortDirection::In) {
            diagnostics.push(error(
                "HNK2104",
                format!(
                    "Connection destination `{}.{}` inside `{}` must be an `in` port.",
                    connection.to_target, connection.to_port, connection.within_component
                ),
            ));
        }

        if from_port.event != to_port.event {
            diagnostics.push(error(
                "HNK2105",
                format!(
                    "Connection `{}::{}` -> `{}::{}` inside `{}` carries event `{}` on the source port, but the destination declares `{}`.",
                    connection.from_target,
                    connection.from_port,
                    connection.to_target,
                    connection.to_port,
                    connection.within_component,
                    from_port.event,
                    to_port.event
                ),
            ));
        }
    }
}

fn validate_transitions(spec: &NormalizedSpec, diagnostics: &mut Vec<Diagnostic>) {
    for component in spec.components.values() {
        for transition in component.transitions.values() {
            for field in &transition.reads {
                if !component.state_fields.contains_key(field) {
                    diagnostics.push(error(
                        "HNK2106",
                        format!(
                            "Transition `{}.{}` reads undefined state field `{field}`.",
                            component.qualified_name, transition.name
                        ),
                    ));
                }
            }
            for field in &transition.writes {
                if !component.state_fields.contains_key(field) {
                    diagnostics.push(error(
                        "HNK2107",
                        format!(
                            "Transition `{}.{}` writes undefined state field `{field}`.",
                            component.qualified_name, transition.name
                        ),
                    ));
                }
            }

            let has_input_port = component
                .ports
                .values()
                .any(|port| matches!(port.direction, PortDirection::In) && port.event == transition.on);
            if !has_input_port {
                diagnostics.push(error(
                    "HNK2108",
                    format!(
                        "Transition `{}.{}` listens to event `{}`, but no `in` port on that component declares it.",
                        component.qualified_name, transition.name, transition.on
                    ),
                ));
            }

            for event in &transition.emits {
                let has_output_port = component
                    .ports
                    .values()
                    .any(|port| matches!(port.direction, PortDirection::Out) && port.event == *event);
                if !has_output_port {
                    diagnostics.push(error(
                        "HNK2109",
                        format!(
                            "Transition `{}.{}` emits event `{event}`, but no `out` port on that component declares it.",
                            component.qualified_name, transition.name
                        ),
                    ));
                }
            }
        }
    }
}

fn validate_persistence(spec: &NormalizedSpec, diagnostics: &mut Vec<Diagnostic>) {
    for component in spec.components.values() {
        if let Some(persistence) = &component.persistence {
            for field in &persistence.durable {
                if !component.state_fields.contains_key(field) {
                    diagnostics.push(error(
                        "HNK2110",
                        format!(
                            "Component `{}` marks undefined state field `{field}` as durable.",
                            component.qualified_name
                        ),
                    ));
                }
            }
        }
    }
}

fn validate_must_reply(spec: &NormalizedSpec, diagnostics: &mut Vec<Diagnostic>) {
    let reverse_connections = reverse_connections(&spec.connections);

    for component in spec.components.values() {
        for port in component.ports.values() {
            if !matches!(port.direction, PortDirection::Out) || !has_must_reply(&port.contracts) {
                continue;
            }

            let requested_event = &port.event;
            if !has_declared_reply_path(
                spec,
                &reverse_connections,
                &component.qualified_name,
                &port.name,
                requested_event,
            ) {
                diagnostics.push(error(
                    "HNK2111",
                    format!(
                        "Port `{}.{}` declares `must_reply` for event `{requested_event}`, but no reply path is declared back to that component.",
                        component.qualified_name, port.name
                    ),
                ));
            }
        }
    }
}

fn has_declared_reply_path(
    spec: &NormalizedSpec,
    reverse_connections: &HashMap<(String, String, String), Vec<&NormalizedConnection>>,
    requester_component: &str,
    requester_port: &str,
    requested_event: &str,
) -> bool {
    let request_scope = requester_component;
    let outbound_matches = spec.connections.iter().filter(|connection| {
        connection.within_component == request_scope
            && connection.from_target == "self"
            && connection.from_port == requester_port
    });

    for connection in outbound_matches {
        let target_component = &spec.components[&connection.to_component];
        for transition in target_component.transitions.values() {
            if transition.on != requested_event {
                continue;
            }
            for emitted_event in &transition.emits {
                let Some(reply_source_ports) = output_ports_for_event(target_component, emitted_event) else {
                    continue;
                };

                for reply_source_port in reply_source_ports {
                    if can_route_event_back(
                        spec,
                        reverse_connections,
                        request_scope,
                        &connection.to_target,
                        reply_source_port,
                        emitted_event,
                    ) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

fn output_ports_for_event<'a>(
    component: &'a NormalizedComponent,
    event: &str,
) -> Option<Vec<&'a str>> {
    let ports: Vec<&str> = component
        .ports
        .values()
        .filter(|port| matches!(port.direction, PortDirection::Out) && port.event == event)
        .map(|port| port.name.as_str())
        .collect();

    if ports.is_empty() {
        None
    } else {
        Some(ports)
    }
}

fn can_route_event_back(
    spec: &NormalizedSpec,
    reverse_connections: &HashMap<(String, String, String), Vec<&NormalizedConnection>>,
    within_component: &str,
    source_target: &str,
    source_port: &str,
    event: &str,
) -> bool {
    let mut queue = VecDeque::from([(
        within_component.to_string(),
        source_target.to_string(),
        source_port.to_string(),
    )]);
    let mut visited = BTreeSet::new();

    while let Some((scope, target, port)) = queue.pop_front() {
        if !visited.insert((scope.clone(), target.clone(), port.clone())) {
            continue;
        }

        let key = (scope.clone(), target.clone(), port.clone());
        let Some(connections) = reverse_connections.get(&key) else {
            continue;
        };

        for connection in connections {
            let target_component = &spec.components[&connection.to_component];
            let target_port = &target_component.ports[&connection.to_port];

            if target_port.event != event {
                continue;
            }

            if connection.to_target == "self" {
                return true;
            }

            for next_port in target_component.ports.values().filter(|port| {
                matches!(port.direction, PortDirection::Out) && port.event == event
            }) {
                queue.push_back((
                    connection.within_component.clone(),
                    connection.to_target.clone(),
                    next_port.name.clone(),
                ));
            }
        }
    }

    false
}

fn reverse_connections<'a>(
    connections: &'a [NormalizedConnection],
) -> HashMap<(String, String, String), Vec<&'a NormalizedConnection>> {
    let mut index = HashMap::<(String, String, String), Vec<&NormalizedConnection>>::new();
    for connection in connections {
        index
            .entry((
                connection.within_component.clone(),
                connection.from_target.clone(),
                connection.from_port.clone(),
            ))
            .or_default()
            .push(connection);
    }
    index
}

fn error(code: &'static str, message: String) -> Diagnostic {
    Diagnostic::new(Severity::Error, code, message, None, None)
}
