use hnk_tools::ir::NormalizedComponent;
use hnk_tools::NormalizedSpec;

use super::{incoming_ports, outgoing_ports};

pub fn render(component: &NormalizedComponent, spec: &NormalizedSpec, header: &str) -> String {
    let mut out = String::new();
    out.push_str(header);
    out.push_str("\n\n");
    out.push_str(&format!(
        "pub const COMPONENT_NAME: &str = {:?};\n\n",
        component.name
    ));

    out.push_str("// inbound ports\n");
    for port in incoming_ports(component) {
        out.push_str(&format!(
            "pub const PORT_{}: &str = {:?};\n",
            port.name.to_ascii_uppercase(),
            port.name
        ));
    }
    out.push_str("\n// outbound ports\n");
    for port in outgoing_ports(component) {
        out.push_str(&format!(
            "pub const PORT_{}: &str = {:?};\n",
            port.name.to_ascii_uppercase(),
            port.name
        ));
    }

    out.push_str("\n// actor boundary notes\n");
    for connection in &spec.actor_boundary_crossings {
        if connection.from_component == component.name || connection.to_component == component.name {
            out.push_str(&format!(
                "// actor-crossing: {}.{} -> {}.{}\n",
                connection.from_component,
                connection.from_port,
                connection.to_component,
                connection.to_port
            ));
        }
    }
    out
}
