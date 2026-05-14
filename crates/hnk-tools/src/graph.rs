use std::fmt::Write;

use hnk_idl::schema::ConnectionLocality;

use crate::ir::NormalizedConnection;
use crate::ir::NormalizedSpec;

pub fn render_mermaid(spec: &NormalizedSpec) -> String {
    let mut out = String::new();
    writeln!(&mut out, "flowchart LR").unwrap();

    for component in spec.components.values() {
        writeln!(
            &mut out,
            "  subgraph component_{}[\"component: {}\"]",
            sanitize(&component.qualified_name),
            component.qualified_name
        )
        .unwrap();
        writeln!(
            &mut out,
            "    node_{}_self[\"self\"]",
            sanitize(&component.qualified_name)
        )
        .unwrap();
        for component_use in component.uses.values() {
            writeln!(
                &mut out,
                "    node_{}_{}[\"{}: {}\"]",
                sanitize(&component.qualified_name),
                sanitize(&component_use.name),
                component_use.name,
                component_use.component
            )
            .unwrap();
        }
        writeln!(&mut out, "  end").unwrap();
    }

    for connection in &spec.connections {
        writeln!(
            &mut out,
            "  node_{}_{} -->|\"{}\"| node_{}_{}",
            sanitize(&connection.within_component),
            sanitize(&connection.from_target),
            edge_label(spec, connection),
            sanitize(&connection.within_component),
            sanitize(&connection.to_target),
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
                component.qualified_name, transition.name, transition.on, emits
            )
            .unwrap();
        }
    }

    out
}

fn edge_label(spec: &NormalizedSpec, connection: &NormalizedConnection) -> String {
    let source_component = if connection.from_target == "self" {
        &spec.components[&connection.within_component]
    } else {
        &spec.components[&connection.from_component]
    };
    let source_port = &source_component.ports[&connection.from_port];
    let locality = match connection.locality {
        ConnectionLocality::Local => "local",
        ConnectionLocality::NonLocal => "non-local",
    };

    format!("{locality}: {}", source_port.event)
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|char| match char {
            'a'..='z' | 'A'..='Z' | '0'..='9' => char,
            _ => '_',
        })
        .collect()
}
