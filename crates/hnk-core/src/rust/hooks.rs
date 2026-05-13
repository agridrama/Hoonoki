use hnk_tools::ir::NormalizedComponent;

use super::{module_name, type_name};

pub fn render(component: &NormalizedComponent, header: &str) -> String {
    let mut out = String::new();
    let type_name = type_name(&component.name);
    out.push_str(header);
    out.push_str("\n\nuse super::state::");
    out.push_str(&format!("{type_name}State;\n"));
    out.push_str("use super::super::runtime_contracts::{HookMetadata, LocalDispatch, MustReplyHooks, NonLocalDispatch, RuntimeHooks, TimeoutHooks};\n\n");
    out.push_str(&format!("pub trait {}Handler {{\n", type_name));
    for transition in component.transitions.values() {
        out.push_str(&format!(
            "    fn handle_{}(&mut self, state: &mut {}State);\n",
            module_name(&transition.name),
            type_name
        ));
    }
    out.push_str("}\n\n");
    out.push_str("pub fn metadata(component: &'static str, port: Option<&'static str>, event: Option<&'static str>, actor_crossing: bool) -> HookMetadata {\n");
    out.push_str("    HookMetadata { component, port, event, actor_crossing, correlation_hint: None, retry_hint: None }\n");
    out.push_str("}\n");
    out
}
