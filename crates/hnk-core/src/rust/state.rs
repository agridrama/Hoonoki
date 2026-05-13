use hnk_tools::ir::NormalizedComponent;

use super::{rust_type, type_name};

pub fn render(component: &NormalizedComponent, header: &str) -> String {
    let mut out = String::new();
    out.push_str(header);
    out.push_str("\n\n");
    out.push_str("#[derive(Debug, Clone, Default)]\n");
    out.push_str(&format!("pub struct {}State {{\n", type_name(&component.name)));
    if component.state_fields.is_empty() {
        out.push_str("    pub _placeholder: (),\n");
    } else {
        for field in component.state_fields.values() {
            out.push_str(&format!(
                "    pub {}: {},\n",
                field.name,
                rust_type(&field.field_type)
            ));
        }
    }
    out.push_str("}\n");
    out
}
