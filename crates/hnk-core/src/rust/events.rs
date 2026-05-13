use hnk_tools::ir::NormalizedComponent;
use hnk_tools::NormalizedSpec;

use super::{component_events, rust_type, type_name};

pub fn render(component: &NormalizedComponent, spec: &NormalizedSpec, header: &str) -> String {
    let mut out = String::new();
    out.push_str(header);
    out.push_str("\n\n");

    for event in component_events(component, spec) {
        out.push_str("#[derive(Debug, Clone, Default)]\n");
        out.push_str(&format!("pub struct {} {{\n", type_name(&event.name)));
        if event.fields.is_empty() {
            out.push_str("    pub _placeholder: (),\n");
        } else {
            for field in &event.fields {
                out.push_str(&format!(
                    "    pub {}: {},\n",
                    field.name,
                    rust_type(&field.field_type)
                ));
            }
        }
        out.push_str("}\n\n");
    }

    out
}
