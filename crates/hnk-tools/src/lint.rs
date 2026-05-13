use std::collections::BTreeSet;

use hnk_idl::DiagnosticSet;
use hnk_idl::ast::OpaqueValue;
use hnk_idl::diagnostics::{Diagnostic, Severity};

use crate::ir::NormalizedSpec;

pub fn lint(spec: &NormalizedSpec) -> DiagnosticSet {
    let mut diagnostics = Vec::new();

    warn_on_unconnected_ports(spec, &mut diagnostics);
    warn_on_unused_state_fields(spec, &mut diagnostics);
    warn_on_durable_state_gaps(spec, &mut diagnostics);

    DiagnosticSet::new(diagnostics)
}

fn warn_on_unconnected_ports(spec: &NormalizedSpec, diagnostics: &mut Vec<Diagnostic>) {
    let mut connected = BTreeSet::new();
    for connection in &spec.connections {
        connected.insert((connection.from_component.clone(), connection.from_port.clone()));
        connected.insert((connection.to_component.clone(), connection.to_port.clone()));
    }

    for component in spec.components.values() {
        for port in component.ports.values() {
            if !connected.contains(&(component.name.clone(), port.name.clone())) {
                diagnostics.push(warning(
                    "HNK2201",
                    format!(
                        "Port `{}.{}` is not connected to any declared propagation path.",
                        component.name, port.name
                    ),
                    Some(
                        "Connect this port or remove it if the interaction is not intended yet."
                            .to_string(),
                    ),
                ));
            }
        }
    }
}

fn warn_on_unused_state_fields(spec: &NormalizedSpec, diagnostics: &mut Vec<Diagnostic>) {
    for component in spec.components.values() {
        let mut referenced = BTreeSet::new();
        for transition in component.transitions.values() {
            referenced.extend(transition.reads.iter().cloned());
            referenced.extend(transition.writes.iter().cloned());
        }

        for field in component.state_fields.values() {
            if !referenced.contains(&field.name) {
                diagnostics.push(warning(
                    "HNK2202",
                    format!(
                        "State field `{}.{}` is never referenced by any transition.",
                        component.name, field.name
                    ),
                    Some(
                        "Remove the field or add a transition that reads or writes it."
                            .to_string(),
                    ),
                ));
            }
        }
    }
}

fn warn_on_durable_state_gaps(spec: &NormalizedSpec, diagnostics: &mut Vec<Diagnostic>) {
    for component in spec.components.values() {
        let expects_recovery = component
            .assumptions
            .iter()
            .any(annotation_mentions_crash_recovery);

        let durable_is_empty = component
            .persistence
            .as_ref()
            .is_none_or(|persistence| persistence.durable.is_empty());

        if expects_recovery && durable_is_empty {
            diagnostics.push(warning(
                "HNK2203",
                format!(
                    "Component `{}` mentions crash recovery assumptions but declares no durable state.",
                    component.name
                ),
                Some(
                    "Add `persistence.durable` fields or remove the recovery assumption if durability is not required."
                        .to_string(),
                ),
            ));
        }
    }
}

fn annotation_mentions_crash_recovery(value: &OpaqueValue) -> bool {
    match value {
        OpaqueValue::String(text) => text.contains("crash_recovery"),
        OpaqueValue::Sequence(values) => values.iter().any(annotation_mentions_crash_recovery),
        OpaqueValue::Mapping(entries) => entries.iter().any(|(key, value)| {
            annotation_mentions_crash_recovery(key) || annotation_mentions_crash_recovery(value)
        }),
        _ => false,
    }
}

fn warning(code: &'static str, message: String, suggestion: Option<String>) -> Diagnostic {
    Diagnostic::new(Severity::Warning, code, message, None, suggestion)
}
