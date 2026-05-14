use hnk_tools::ir::NormalizedComponent;
use hnk_tools::NormalizedSpec;

use super::{incoming_ports, outgoing_ports};

pub fn render(component: &NormalizedComponent, spec: &NormalizedSpec, header: &str) -> String {
    let mut out = String::new();
    out.push_str(header);
    out.push_str("\n\n");
    out.push_str(&format!(
        "pub const COMPONENT_NAME: &str = {:?};\n\n",
        component.qualified_name
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

    out.push_str("\n// non-local wiring notes\n");
    for connection in &spec.non_local_connections {
        if connection.within_component == component.qualified_name {
            out.push_str(&format!(
                "// non-local: {}.{} -> {}.{}\n",
                connection.from_target,
                connection.from_port,
                connection.to_target,
                connection.to_port
            ));
        }
    }
    out
}
