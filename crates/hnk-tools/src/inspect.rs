use std::fmt::Write;

use crate::ir::NormalizedSpec;

pub fn render_inspect_summary(spec: &NormalizedSpec) -> String {
    let mut out = String::new();
    writeln!(&mut out, "Spec version: {}", spec.version).unwrap();
    writeln!(
        &mut out,
        "Overview: {} events, {} components, {} actors, {} connections",
        spec.events.len(),
        spec.components.len(),
        spec.actors.len(),
        spec.connections.len()
    )
    .unwrap();

    writeln!(&mut out, "\nConnections:").unwrap();
    let local = spec
        .connections
        .iter()
        .filter(|connection| !connection.crosses_actor_boundary)
        .count();
    let crossing = spec.actor_boundary_crossings.len();
    writeln!(&mut out, "- local: {local}").unwrap();
    writeln!(&mut out, "- actor-crossing: {crossing}").unwrap();

    if !spec.actor_boundary_crossings.is_empty() {
        writeln!(&mut out, "\nActor-crossing details:").unwrap();
        for connection in &spec.actor_boundary_crossings {
            writeln!(
                &mut out,
                "- {}.{} -> {}.{} ({} -> {})",
                connection.from_component,
                connection.from_port,
                connection.to_component,
                connection.to_port,
                connection.from_actor.as_deref().unwrap_or("<unassigned>"),
                connection.to_actor.as_deref().unwrap_or("<unassigned>")
            )
            .unwrap();
        }
    }

    writeln!(&mut out, "\nComponents:").unwrap();
    for component in spec.components.values() {
        writeln!(
            &mut out,
            "- {}: {} ports, {} transitions, {} state fields",
            component.name,
            component.ports.len(),
            component.transitions.len(),
            component.state_fields.len()
        )
        .unwrap();
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

        if !component.state_fields.is_empty() {
            writeln!(&mut out, "  state-owning component: yes").unwrap();
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

        let contract_ports: Vec<String> = component
            .ports
            .values()
            .filter(|port| !port.contracts.is_empty())
            .map(|port| format!("{}.{}", component.name, port.name))
            .collect();
        writeln!(
            &mut out,
            "  contract-bearing ports: {}",
            display_list(&contract_ports)
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
