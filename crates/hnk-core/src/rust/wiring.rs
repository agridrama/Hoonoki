use hnk_idl::schema::{ConnectionLocality, PortDirection};
use hnk_tools::contracts::{has_must_reply, timeout_after};
use hnk_tools::ir::NormalizedComponent;
use hnk_tools::NormalizedSpec;

use super::{module_name, outgoing_connections_for_port, outgoing_ports, type_name};

pub fn render(component: &NormalizedComponent, spec: &NormalizedSpec, header: &str) -> String {
    let mut out = String::new();
    let type_name = type_name(&component.name);
    out.push_str(header);
    out.push_str("\n\nuse super::hooks::{metadata, ");
    out.push_str(&format!("{type_name}Handler"));
    out.push_str("};\n");
    out.push_str(&format!("use super::state::{}State;\n", type_name));
    out.push_str("use super::super::runtime_contracts::{LocalDispatch, MustReplyHooks, NonLocalDispatch, RuntimeHooks, TimeoutHooks};\n\n");

    out.push_str(&format!(
        "pub struct {}Wiring<'a, H, R, M, T, L, N> {{\n    pub handler: &'a mut H,\n    pub runtime_hooks: &'a mut R,\n    pub must_reply_hooks: &'a mut M,\n    pub timeout_hooks: &'a mut T,\n    pub local_dispatch: &'a mut L,\n    pub non_local_dispatch: &'a mut N,\n}}\n\n",
        type_name
    ));
    out.push_str(&format!(
        "impl<'a, H, R, M, T, L, N> {}Wiring<'a, H, R, M, T, L, N>\nwhere\n    H: {}Handler,\n    R: RuntimeHooks,\n    M: MustReplyHooks,\n    T: TimeoutHooks,\n    L: LocalDispatch,\n    N: NonLocalDispatch,\n{{\n",
        type_name, type_name
    ));
    out.push_str(&format!(
        "    pub fn run_transition(&mut self, transition: &str, state: &mut {}State) {{\n        let meta = metadata({:?}, None, None, \"local\");\n        self.runtime_hooks.on_transition_enter(&meta);\n        match transition {{\n",
        type_name, component.qualified_name
    ));
    for transition in component.transitions.values() {
        out.push_str(&format!(
            "            {:?} => self.handler.handle_{}(state),\n",
            transition.name,
            module_name(&transition.name)
        ));
    }
    out.push_str("            _ => {}\n        }\n        self.runtime_hooks.on_transition_exit(&meta);\n    }\n\n");

    for port in outgoing_ports(component) {
        let method_name = module_name(&port.name);
        out.push_str(&format!(
            "    pub fn dispatch_{}(&mut self) {{\n",
            method_name
        ));
        let must_reply = has_must_reply(&port.contracts);
        let timeout = timeout_after(&port.contracts);
        let event = &port.event;
        out.push_str(&format!(
            "        let meta_{} = metadata({:?}, Some({:?}), Some({:?}), \"local\");\n        self.runtime_hooks.on_send(&meta_{});\n",
            module_name(event),
            component.qualified_name,
            port.name,
            event,
            module_name(event)
        ));
        if must_reply {
            out.push_str(&format!(
                "        self.must_reply_hooks.on_request_started(&meta_{});\n",
                module_name(event)
            ));
            out.push_str(&format!(
                "        // If runtime determines the reply was never fulfilled, call:\n        // self.must_reply_hooks.on_reply_missing(&meta_{});\n",
                module_name(event)
            ));
        }
        if let Some(after) = &timeout {
            out.push_str(&format!(
                "        self.timeout_hooks.on_timeout_window_started(&meta_{}, {:?});\n        // If runtime observes timeout violation, call:\n        // self.timeout_hooks.on_timeout_contract_violation(&meta_{}, {:?});\n",
                module_name(event), after, module_name(event), after
            ));
        }

        let connections = outgoing_connections_for_port(spec, &component.qualified_name, &port.name);
        if connections.is_empty() {
            out.push_str("        // No declared propagation paths for this port.\n");
        }
        for connection in connections {
            if matches!(connection.locality, ConnectionLocality::NonLocal) {
                out.push_str(&format!(
                    "        let boundary_meta = metadata({:?}, Some({:?}), Some({:?}), \"non_local\");\n        self.runtime_hooks.on_non_local_dispatch(&boundary_meta);\n        self.non_local_dispatch.dispatch_non_local(&boundary_meta);\n        // target endpoint: {}.{}\n",
                    component.qualified_name,
                    port.name,
                    event,
                    connection.to_target,
                    connection.to_port
                ));
            } else {
                out.push_str(&format!(
                    "        let local_meta = metadata({:?}, Some({:?}), Some({:?}), \"local\");\n        self.local_dispatch.dispatch_local(&local_meta);\n        // target endpoint: {}.{}\n",
                    component.qualified_name,
                    port.name,
                    event,
                    connection.to_target,
                    connection.to_port
                ));
            }
        }
        out.push_str("    }\n\n");
    }

    for port in component
        .ports
        .values()
        .filter(|port| matches!(port.direction, PortDirection::In))
    {
        let event = &port.event;
        out.push_str(&format!(
            "    pub fn observe_{}_{}(&mut self) {{\n        let meta = metadata({:?}, Some({:?}), Some({:?}), \"local\");\n        self.runtime_hooks.on_receive(&meta);\n",
            method_name(&port.name),
            module_name(event),
            component.qualified_name,
            port.name,
            event
        ));
        let lowered = event.to_ascii_lowercase();
        if lowered.contains("timeout") || lowered.contains("timedout") {
            out.push_str("        self.timeout_hooks.on_timeout_event_fired(&meta);\n");
        }
        if lowered.contains("reply") || lowered.contains("returned") {
            out.push_str("        self.must_reply_hooks.on_reply_observed(&meta);\n");
        }
        out.push_str("    }\n\n");
    }

    out.push_str("}\n");
    out
}

fn method_name(name: &str) -> String {
    module_name(name)
}
