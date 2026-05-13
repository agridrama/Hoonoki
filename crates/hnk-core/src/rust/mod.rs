mod events;
mod hooks;
mod persistence;
mod ports;
mod state;
mod wiring;

use std::path::PathBuf;

use hnk_idl::schema::PortDirection;
use hnk_tools::contracts::{has_must_reply, timeout_after};
use hnk_tools::ir::{
    NormalizedComponent, NormalizedConnection, NormalizedEvent, NormalizedPort, NormalizedSpec,
};

pub struct RenderedFile {
    pub relative_path: PathBuf,
    pub contents: String,
    pub preserve_existing: bool,
}

pub fn render_project_files(
    spec: &NormalizedSpec,
    header: &str,
    create_app_scaffold: bool,
) -> Vec<RenderedFile> {
    let mut files = Vec::new();

    files.push(RenderedFile {
        relative_path: PathBuf::from("src/generated/mod.rs"),
        contents: render_generated_root_mod(spec, header),
        preserve_existing: false,
    });
    files.push(RenderedFile {
        relative_path: PathBuf::from("src/generated/runtime_contracts.rs"),
        contents: render_generated_runtime_contracts(header),
        preserve_existing: false,
    });

    for component in spec.components.values() {
        let module_name = module_name(&component.name);
        let component_dir = format!("src/generated/{module_name}");
        files.push(RenderedFile {
            relative_path: PathBuf::from(format!("{component_dir}/mod.rs")),
            contents: render_component_mod(component, header),
            preserve_existing: false,
        });
        files.push(RenderedFile {
            relative_path: PathBuf::from(format!("{component_dir}/events.rs")),
            contents: events::render(component, spec, header),
            preserve_existing: false,
        });
        files.push(RenderedFile {
            relative_path: PathBuf::from(format!("{component_dir}/state.rs")),
            contents: state::render(component, header),
            preserve_existing: false,
        });
        files.push(RenderedFile {
            relative_path: PathBuf::from(format!("{component_dir}/ports.rs")),
            contents: ports::render(component, spec, header),
            preserve_existing: false,
        });
        files.push(RenderedFile {
            relative_path: PathBuf::from(format!("{component_dir}/hooks.rs")),
            contents: hooks::render(component, header),
            preserve_existing: false,
        });
        files.push(RenderedFile {
            relative_path: PathBuf::from(format!("{component_dir}/persistence.rs")),
            contents: persistence::render(component, header),
            preserve_existing: false,
        });
        files.push(RenderedFile {
            relative_path: PathBuf::from(format!("{component_dir}/wiring.rs")),
            contents: wiring::render(component, spec, header),
            preserve_existing: false,
        });

        if create_app_scaffold {
            files.push(RenderedFile {
                relative_path: PathBuf::from(format!("src/app/{module_name}.rs")),
                contents: render_app_scaffold(component, header),
                preserve_existing: true,
            });
        }
    }

    files
}

pub fn module_name(name: &str) -> String {
    let mut output = String::new();
    let mut last_was_lower = false;
    for ch in name.chars() {
        if ch.is_ascii_uppercase() {
            if last_was_lower {
                output.push('_');
            }
            output.push(ch.to_ascii_lowercase());
            last_was_lower = false;
        } else if ch.is_ascii_alphanumeric() {
            output.push(ch.to_ascii_lowercase());
            last_was_lower = ch.is_ascii_lowercase() || ch.is_ascii_digit();
        } else {
            if !output.ends_with('_') {
                output.push('_');
            }
            last_was_lower = false;
        }
    }
    output
}

pub fn type_name(name: &str) -> String {
    name.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

pub fn rust_type(type_name: &str) -> String {
    match type_name.replace(' ', "").as_str() {
        "string" => "String".to_string(),
        "bool" => "bool".to_string(),
        "i64" => "i64".to_string(),
        "u64" => "u64".to_string(),
        "map<string,string>" => "std::collections::BTreeMap<String, String>".to_string(),
        _ => "String".to_string(),
    }
}

pub fn component_events<'a>(
    component: &'a NormalizedComponent,
    spec: &'a NormalizedSpec,
) -> Vec<&'a NormalizedEvent> {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for event_name in component
        .inbound_events
        .iter()
        .chain(component.outbound_events.iter())
    {
        if seen.insert(event_name.clone()) {
            if let Some(event) = spec.events.get(event_name) {
                out.push(event);
            }
        }
    }
    out
}

pub fn outgoing_ports<'a>(component: &'a NormalizedComponent) -> Vec<&'a NormalizedPort> {
    component
        .ports
        .values()
        .filter(|port| matches!(port.direction, PortDirection::Out))
        .collect()
}

pub fn incoming_ports<'a>(component: &'a NormalizedComponent) -> Vec<&'a NormalizedPort> {
    component
        .ports
        .values()
        .filter(|port| matches!(port.direction, PortDirection::In))
        .collect()
}

pub fn outgoing_connections_for_port<'a>(
    spec: &'a NormalizedSpec,
    component_name: &str,
    port_name: &str,
) -> Vec<&'a NormalizedConnection> {
    spec.connections
        .iter()
        .filter(|connection| {
            connection.from_component == component_name && connection.from_port == port_name
        })
        .collect()
}

pub fn render_contract_metadata(port: &NormalizedPort) -> String {
    let must_reply = has_must_reply(&port.contracts);
    let timeout = timeout_after(&port.contracts);
    match (must_reply, timeout.as_deref()) {
        (false, None) => "None".to_string(),
        _ => format!(
            "Some(({}, {:?}))",
            must_reply,
            timeout.as_deref().unwrap_or("")
        ),
    }
}

fn render_generated_root_mod(spec: &NormalizedSpec, header: &str) -> String {
    let mut out = String::new();
    out.push_str(header);
    out.push('\n');
    out.push('\n');
    out.push_str("pub mod runtime_contracts;\n");
    for component in spec.components.values() {
        out.push_str(&format!("pub mod {};\n", module_name(&component.name)));
    }
    out
}

fn render_generated_runtime_contracts(header: &str) -> String {
    let mut out = String::new();
    out.push_str(header);
    out.push_str(
        "\n\n#[derive(Debug, Clone)]\n\
         pub struct HookMetadata {\n\
             pub component: &'static str,\n\
             pub port: Option<&'static str>,\n\
             pub event: Option<&'static str>,\n\
             pub actor_crossing: bool,\n\
             pub correlation_hint: Option<&'static str>,\n\
             pub retry_hint: Option<&'static str>,\n\
         }\n\n\
         pub trait RuntimeHooks {\n\
             fn on_send(&mut self, metadata: &HookMetadata);\n\
             fn on_receive(&mut self, metadata: &HookMetadata);\n\
             fn on_transition_enter(&mut self, metadata: &HookMetadata);\n\
             fn on_transition_exit(&mut self, metadata: &HookMetadata);\n\
             fn on_non_local_dispatch(&mut self, metadata: &HookMetadata);\n\
         }\n\n\
         pub trait MustReplyHooks {\n\
             fn on_request_started(&mut self, metadata: &HookMetadata);\n\
             fn on_reply_observed(&mut self, metadata: &HookMetadata);\n\
             fn on_reply_missing(&mut self, metadata: &HookMetadata);\n\
         }\n\n\
         pub trait TimeoutHooks {\n\
             fn on_timeout_window_started(&mut self, metadata: &HookMetadata, after: &str);\n\
             fn on_timeout_event_fired(&mut self, metadata: &HookMetadata);\n\
             fn on_timeout_contract_violation(&mut self, metadata: &HookMetadata, after: &str);\n\
         }\n\n\
         pub trait LocalDispatch {\n\
             fn dispatch_local(&mut self, metadata: &HookMetadata);\n\
         }\n\n\
         pub trait NonLocalDispatch {\n\
             fn dispatch_non_local(&mut self, metadata: &HookMetadata);\n\
         }\n",
    );
    out
}

fn render_component_mod(component: &NormalizedComponent, header: &str) -> String {
    let mut out = String::new();
    out.push_str(header);
    out.push('\n');
    out.push_str("\npub mod events;\npub mod hooks;\npub mod persistence;\npub mod ports;\npub mod state;\npub mod wiring;\n");
    out.push_str(&format!(
        "\npub use hooks::{}Handler;\npub use state::{}State;\n",
        type_name(&component.name),
        type_name(&component.name)
    ));
    out
}

fn render_app_scaffold(component: &NormalizedComponent, header: &str) -> String {
    let type_name = type_name(&component.name);
    let component_module = module_name(&component.name);
    format!(
        "{header}\n\nuse crate::generated::{component_module}::hooks::{type_name}Handler;\nuse crate::generated::{component_module}::state::{type_name}State;\n\npub struct {type_name}App;\n\nimpl {type_name}Handler for {type_name}App {{\n{methods}}}\n",
        methods = component
            .transitions
            .values()
            .map(|transition| format!(
                "    fn handle_{}(&mut self, _state: &mut {}State) {{\n        // TODO: implement transition logic for `{}`.\n    }}\n",
                module_name(&transition.name),
                type_name,
                transition.name
            ))
            .collect::<String>()
    )
}
