use std::fmt::Write;

use crate::ir::{NormalizedConnection, NormalizedSpec};

pub fn render_mermaid(spec: &NormalizedSpec) -> String {
    let mut out = String::new();
    writeln!(&mut out, "flowchart LR").unwrap();

    for actor in spec.actors.values() {
        writeln!(&mut out, "  subgraph actor_{}[\"actor: {}\"]", sanitize(&actor.name), actor.name)
            .unwrap();
        for component_name in &actor.components {
            if let Some(component) = spec.components.get(component_name) {
                writeln!(
                    &mut out,
                    "    comp_{}[\"{}\"]",
                    sanitize(&component.name),
                    component.name
                )
                .unwrap();
            }
        }
        writeln!(&mut out, "  end").unwrap();
    }

    for component in spec.components.values() {
        if !spec.actors.values().any(|actor| actor.components.contains(&component.name)) {
            writeln!(
                &mut out,
                "  comp_{}[\"{}\"]",
                sanitize(&component.name),
                component.name
            )
            .unwrap();
        }
    }

    for connection in &spec.connections {
        writeln!(
            &mut out,
            "  comp_{} -->|\"{}\"| comp_{}",
            sanitize(&connection.from_component),
            edge_label(spec, connection),
            sanitize(&connection.to_component),
        )
        .unwrap();
    }

    writeln!(&mut out, "  %% transition adjacency summary").unwrap();
    for component in spec.components.values() {
        for transition in component.transitions.values() {
            let emits = if transition.emits.is_empty() {
                "(no emits)".to_string()
            } else {
                transition.emits.join(", ")
            };
            writeln!(
                &mut out,
                "  %% {}.{}: on {} -> {}",
                component.name, transition.name, transition.on, emits
            )
            .unwrap();
        }
    }

    out
}

fn edge_label(spec: &NormalizedSpec, connection: &NormalizedConnection) -> String {
    let source_port = &spec.components[&connection.from_component].ports[&connection.from_port];
    let actor_label = if connection.crosses_actor_boundary {
        "cross-actor"
    } else {
        "local"
    };

    format!("{actor_label}: {}", source_port.event)
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|char| match char {
            'a'..='z' | 'A'..='Z' | '0'..='9' => char,
            _ => '_',
        })
        .collect()
}
