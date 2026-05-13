use hnk_tools::ir::NormalizedComponent;

use super::type_name;

pub fn render(component: &NormalizedComponent, header: &str) -> String {
    let mut out = String::new();
    out.push_str(header);
    out.push_str("\n\n");
    out.push_str(&format!(
        "pub trait {}Persistence {{\n",
        type_name(&component.name)
    ));
    out.push_str("    fn load_state(&mut self) -> Result<(), String>;\n");
    out.push_str("    fn save_state(&mut self) -> Result<(), String>;\n");
    out.push_str("}\n\n");

    if let Some(persistence) = &component.persistence {
        out.push_str("// durable state fields\n");
        for field in &persistence.durable {
            out.push_str(&format!("// - {field}\n"));
        }
    } else {
        out.push_str("// no durable state declared\n");
    }
    out
}
