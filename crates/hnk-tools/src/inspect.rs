use std::fmt::Write;

use hnk_idl::schema::ConnectionLocality;

use crate::ir::NormalizedSpec;

pub fn render_inspect_summary(spec: &NormalizedSpec) -> String {
    let mut out = String::new();
    writeln!(&mut out, "Spec version: {}", spec.version).unwrap();
    writeln!(&mut out, "Entry package: {}", spec.entry_package).unwrap();
    writeln!(
        &mut out,
        "Overview: {} events, {} components, {} connections",
        spec.events.len(),
        spec.components.len(),
        spec.connections.len()
    )
    .unwrap();

    let local = spec
        .connections
        .iter()
        .filter(|connection| matches!(connection.locality, ConnectionLocality::Local))
        .count();
    let non_local = spec.non_local_connections.len();
    writeln!(&mut out, "\nConnections:").unwrap();
    writeln!(&mut out, "- local: {local}").unwrap();
    writeln!(&mut out, "- non-local: {non_local}").unwrap();

    if !spec.non_local_connections.is_empty() {
        writeln!(&mut out, "\nNon-local details:").unwrap();
        for connection in &spec.non_local_connections {
            writeln!(
                &mut out,
                "- within {}: {}.{} -> {}.{}",
                connection.within_component,
                connection.from_target,
                connection.from_port,
                connection.to_target,
                connection.to_port
            )
            .unwrap();
        }
    }

    writeln!(&mut out, "\nComponents:").unwrap();
    for component in spec.components.values() {
        writeln!(
            &mut out,
            "- {}: {} uses, {} ports, {} transitions, {} state fields",
            component.qualified_name,
            component.uses.len(),
            component.ports.len(),
            component.transitions.len(),
            component.state_fields.len()
        )
        .unwrap();
        let uses: Vec<String> = component
            .uses
            .values()
            .map(|component_use| format!("{}: {}", component_use.name, component_use.component))
            .collect();
        writeln!(&mut out, "  uses: {}", display_list(&uses)).unwrap();
        writeln!(
            &mut out,
            "  inbound events: {}",
            display_list(&component.inbound_events)
        )
        .unwrap();
        writeln!(
            &mut out,
            "  outbound events: {}",
            display_list(&component.outbound_events)
        )
        .unwrap();
        if let Some(persistence) = &component.persistence {
            writeln!(
                &mut out,
                "  durable state: {}",
                display_list(&persistence.durable)
            )
            .unwrap();
        } else {
            writeln!(&mut out, "  durable state: <none>").unwrap();
        }

        for transition in component.transitions.values() {
            writeln!(
                &mut out,
                "  transition {}: on {} -> {}",
                transition.name,
                transition.on,
                if transition.emits.is_empty() {
                    "(no emits)".to_string()
                } else {
                    transition.emits.join(", ")
                }
            )
            .unwrap();
        }

        let local_wiring = spec
            .connections
            .iter()
            .filter(|connection| {
                connection.within_component == component.qualified_name
                    && matches!(connection.locality, ConnectionLocality::Local)
            })
            .count();
        let non_local_wiring = spec
            .connections
            .iter()
            .filter(|connection| {
                connection.within_component == component.qualified_name
                    && matches!(connection.locality, ConnectionLocality::NonLocal)
            })
            .count();
        writeln!(
            &mut out,
                "  wiring: {} local, {} non-local",
            local_wiring, non_local_wiring
        )
        .unwrap();
    }

    out
}

fn display_list(items: &[String]) -> String {
    if items.is_empty() {
        "<none>".to_string()
    } else {
        items.join(", ")
    }
}
