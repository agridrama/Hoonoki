use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use hnk_idl::ast::{Component, Event};
use hnk_idl::diagnostics::{Diagnostic, DiagnosticSet, Severity};
use hnk_idl::{BundleAst, BundleFileAst, Spec};

use crate::ir::{
    NormalizedComponent, NormalizedComponentUse, NormalizedConnection, NormalizedEvent,
    NormalizedField, NormalizedPersistence, NormalizedPort, NormalizedSpec, NormalizedStateField,
    NormalizedTransition,
};

pub fn normalize(spec: &Spec) -> Result<NormalizedSpec, DiagnosticSet> {
    let inline_path = PathBuf::from("<inline>");
    let bundle = BundleAst {
        entry: inline_path.clone(),
        import_root: inline_path.clone(),
        files: vec![BundleFileAst {
            path: inline_path,
            package: spec.package.clone(),
            imports: Vec::new(),
            spec: spec.clone(),
        }],
        import_graph: BTreeMap::new(),
    };

    normalize_bundle(&bundle)
}

pub fn normalize_bundle(bundle: &BundleAst) -> Result<NormalizedSpec, DiagnosticSet> {
    let mut diagnostics = Vec::new();

    let Some(entry_file) = bundle.files.iter().find(|file| file.path == bundle.entry) else {
        return Err(DiagnosticSet::singleton(error(
            "HNK2000",
            "The bundle entry file was not found in the loaded import graph.".to_string(),
        )));
    };

    for file in &bundle.files {
        if file.spec.version != entry_file.spec.version {
            diagnostics.push(error(
                "HNK2000",
                format!(
                    "Imported file `{}` uses version `{}`, but the entry file uses `{}`.",
                    file.path.display(),
                    file.spec.version,
                    entry_file.spec.version
                ),
            ));
        }
    }

    let event_decls = collect_event_decls(bundle, &mut diagnostics);
    let component_decls = collect_component_decls(bundle, &mut diagnostics);
    let events = build_events(&event_decls);
    let components = build_components(&component_decls, &event_decls, &mut diagnostics);
    let connections = build_connections(bundle, &components, &mut diagnostics);

    if !diagnostics.is_empty() {
        return Err(DiagnosticSet::new(diagnostics));
    }

    let non_local_connections = connections
        .iter()
        .filter(|connection| matches!(connection.locality, hnk_idl::schema::ConnectionLocality::NonLocal))
        .cloned()
        .collect();

    Ok(NormalizedSpec {
        version: entry_file.spec.version.clone(),
        entry_package: entry_file.package.clone(),
        events,
        components,
        connections,
        non_local_connections,
        imports: entry_file.spec.imports.clone(),
        import_graph: bundle.import_graph.clone(),
    })
}

#[derive(Clone, Copy)]
struct EventDecl<'a> {
    package: &'a str,
    path: &'a PathBuf,
    event: &'a Event,
}

#[derive(Clone, Copy)]
struct ComponentDecl<'a> {
    package: &'a str,
    path: &'a PathBuf,
    component: &'a Component,
}

fn collect_event_decls<'a>(
    bundle: &'a BundleAst,
    diagnostics: &mut Vec<Diagnostic>,
) -> BTreeMap<String, EventDecl<'a>> {
    let mut events = BTreeMap::new();

    for file in &bundle.files {
        for event in &file.spec.events {
            let qualified_name = qualify(&file.package, &event.name);
            if events.contains_key(&qualified_name) {
                diagnostics.push(error(
                    "HNK2001",
                    format!(
                        "Duplicate event name `{qualified_name}`. Event names must be unique within a package."
                    ),
                ));
                continue;
            }

            events.insert(
                qualified_name,
                EventDecl {
                    package: &file.package,
                    path: &file.path,
                    event,
                },
            );
        }
    }

    events
}

fn collect_component_decls<'a>(
    bundle: &'a BundleAst,
    diagnostics: &mut Vec<Diagnostic>,
) -> BTreeMap<String, ComponentDecl<'a>> {
    let mut components = BTreeMap::new();

    for file in &bundle.files {
        for component in &file.spec.components {
            let qualified_name = qualify(&file.package, &component.name);
            if components.contains_key(&qualified_name) {
                diagnostics.push(error(
                    "HNK2003",
                    format!(
                        "Duplicate component name `{qualified_name}`. Component names must be unique within a package."
                    ),
                ));
                continue;
            }

            components.insert(
                qualified_name,
                ComponentDecl {
                    package: &file.package,
                    path: &file.path,
                    component,
                },
            );
        }
    }

    components
}

fn build_events(event_decls: &BTreeMap<String, EventDecl<'_>>) -> BTreeMap<String, NormalizedEvent> {
    event_decls
        .iter()
        .map(|(qualified_name, decl)| {
            (
                qualified_name.clone(),
                NormalizedEvent {
                    package: decl.package.to_string(),
                    name: decl.event.name.clone(),
                    qualified_name: qualified_name.clone(),
                    source_path: decl.path.clone(),
                    fields: decl
                        .event
                        .fields
                        .iter()
                        .map(|field| NormalizedField {
                            name: field.name.clone(),
                            field_type: field.field_type.clone(),
                            doc: field.doc.clone(),
                        })
                        .collect(),
                    visibility: decl.event.visibility.clone(),
                    version: decl.event.version.clone(),
                    deprecated_since: decl.event.deprecated.as_ref().and_then(|it| it.since.clone()),
                    deprecated_note: decl.event.deprecated.as_ref().and_then(|it| it.note.clone()),
                    doc: decl.event.doc.clone(),
                    kind: decl.event.kind.clone(),
                    contracts: decl.event.contracts.clone(),
                },
            )
        })
        .collect()
}

fn build_components(
    component_decls: &BTreeMap<String, ComponentDecl<'_>>,
    event_decls: &BTreeMap<String, EventDecl<'_>>,
    diagnostics: &mut Vec<Diagnostic>,
) -> BTreeMap<String, NormalizedComponent> {
    let mut components = BTreeMap::new();

    for (qualified_name, decl) in component_decls {
        let component = decl.component;
        let mut uses = BTreeMap::new();
        for component_use in &component.uses {
            if uses.contains_key(&component_use.name) {
                diagnostics.push(error(
                    "HNK2002",
                    format!(
                        "Component `{}` defines subcomponent instance `{}` more than once.",
                        qualified_name, component_use.name
                    ),
                ));
                continue;
            }

            let Some(resolved_component) = resolve_component_name(
                decl.package,
                &component_use.component,
                component_decls,
                "used component",
                diagnostics,
            ) else {
                continue;
            };

            uses.insert(
                component_use.name.clone(),
                NormalizedComponentUse {
                    name: component_use.name.clone(),
                    component: resolved_component,
                    doc: component_use.doc.clone(),
                },
            );
        }

        let mut ports = BTreeMap::new();
        let mut inbound_events = BTreeSet::new();
        let mut outbound_events = BTreeSet::new();

        for port in &component.ports {
            if ports.contains_key(&port.name) {
                diagnostics.push(error(
                    "HNK2004",
                    format!(
                        "Component `{}` defines port `{}` more than once.",
                        qualified_name, port.name
                    ),
                ));
                continue;
            }

            let Some(event_name) = resolve_event_name(
                decl.package,
                &port.event,
                event_decls,
                "port event",
                diagnostics,
            ) else {
                continue;
            };

            match port.direction {
                hnk_idl::schema::PortDirection::In => {
                    inbound_events.insert(event_name.clone());
                }
                hnk_idl::schema::PortDirection::Out => {
                    outbound_events.insert(event_name.clone());
                }
            }

            ports.insert(
                port.name.clone(),
                NormalizedPort {
                    component_name: qualified_name.clone(),
                    name: port.name.clone(),
                    qualified_name: format!("{qualified_name}.{}", port.name),
                    direction: port.direction.clone(),
                    event: event_name,
                    contracts: port.contracts.clone(),
                    doc: port.doc.clone(),
                },
            );
        }

        let mut state_fields = BTreeMap::new();
        for field in &component.state.fields {
            if state_fields.contains_key(&field.name) {
                diagnostics.push(error(
                    "HNK2006",
                    format!(
                        "Component `{}` defines state field `{}` more than once.",
                        qualified_name, field.name
                    ),
                ));
                continue;
            }

            state_fields.insert(
                field.name.clone(),
                NormalizedStateField {
                    component_name: qualified_name.clone(),
                    name: field.name.clone(),
                    field_type: field.field_type.clone(),
                    doc: field.doc.clone(),
                },
            );
        }

        let mut transitions = BTreeMap::new();
        for transition in &component.transitions {
            if transitions.contains_key(&transition.name) {
                diagnostics.push(error(
                    "HNK2007",
                    format!(
                        "Component `{}` defines transition `{}` more than once.",
                        qualified_name, transition.name
                    ),
                ));
                continue;
            }

            let Some(on) = resolve_event_name(
                decl.package,
                &transition.on,
                event_decls,
                "transition input event",
                diagnostics,
            ) else {
                continue;
            };

            let emits = transition
                .emits
                .iter()
                .filter_map(|event_name| {
                    resolve_event_name(
                        decl.package,
                        event_name,
                        event_decls,
                        "emitted event",
                        diagnostics,
                    )
                })
                .collect();

            transitions.insert(
                transition.name.clone(),
                NormalizedTransition {
                    component_name: qualified_name.clone(),
                    name: transition.name.clone(),
                    on,
                    emits,
                    reads: transition.reads.clone(),
                    writes: transition.writes.clone(),
                    requires: transition.requires.clone(),
                    ensures: transition.ensures.clone(),
                    doc: transition.doc.clone(),
                },
            );
        }

        components.insert(
            qualified_name.clone(),
            NormalizedComponent {
                package: decl.package.to_string(),
                name: component.name.clone(),
                qualified_name: qualified_name.clone(),
                source_path: decl.path.clone(),
                uses,
                ports,
                state_fields,
                transitions,
                persistence: component.persistence.as_ref().map(|persistence| NormalizedPersistence {
                    durable: persistence.durable.clone(),
                    doc: persistence.doc.clone(),
                }),
                assumptions: component.assumptions.clone(),
                doc: component.doc.clone(),
                inbound_events: inbound_events.into_iter().collect(),
                outbound_events: outbound_events.into_iter().collect(),
            },
        );
    }

    components
}

fn build_connections(
    bundle: &BundleAst,
    components: &BTreeMap<String, NormalizedComponent>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<NormalizedConnection> {
    let mut connections = Vec::new();

    for file in &bundle.files {
        for connection in &file.spec.connections {
            let Some(within_component_name) = resolve_component_name(
                &file.package,
                &connection.within,
                components,
                "enclosing component",
                diagnostics,
            ) else {
                continue;
            };

            let Some(within_component) = components.get(&within_component_name) else {
                diagnostics.push(error(
                    "HNK2011",
                    format!(
                        "Connection references undefined enclosing component `{}` in `within`.",
                        connection.within
                    ),
                ));
                continue;
            };

            let Some((from_target, from_port)) = parse_endpoint(&connection.from) else {
                diagnostics.push(error(
                    "HNK2012",
                    format!(
                        "Connection `from` endpoint `{}` must use `self.port` or `instance.port` format.",
                        connection.from
                    ),
                ));
                continue;
            };
            let Some((to_target, to_port)) = parse_endpoint(&connection.to) else {
                diagnostics.push(error(
                    "HNK2013",
                    format!(
                        "Connection `to` endpoint `{}` must use `self.port` or `instance.port` format.",
                        connection.to
                    ),
                ));
                continue;
            };

            let Some((from_component_name, from_component_model)) =
                resolve_target(within_component, components, from_target, diagnostics, "from")
            else {
                continue;
            };
            let Some((to_component_name, to_component_model)) =
                resolve_target(within_component, components, to_target, diagnostics, "to")
            else {
                continue;
            };

            if !from_component_model.ports.contains_key(from_port) {
                diagnostics.push(error(
                    "HNK2014",
                    format!(
                        "Connection `from` references undefined port `{from_target}.{from_port}` within component `{}`.",
                        within_component.qualified_name
                    ),
                ));
                continue;
            }
            if !to_component_model.ports.contains_key(to_port) {
                diagnostics.push(error(
                    "HNK2015",
                    format!(
                        "Connection `to` references undefined port `{to_target}.{to_port}` within component `{}`.",
                        within_component.qualified_name
                    ),
                ));
                continue;
            }

            connections.push(NormalizedConnection {
                within_component: within_component_name,
                from_target: from_target.to_string(),
                from_component: from_component_name,
                from_port: from_port.to_string(),
                to_target: to_target.to_string(),
                to_component: to_component_name,
                to_port: to_port.to_string(),
                locality: connection.locality.clone(),
                contracts: connection.contracts.clone(),
                doc: connection.doc.clone(),
            });
        }
    }

    connections
}

fn resolve_target<'a>(
    within_component: &'a NormalizedComponent,
    components: &'a BTreeMap<String, NormalizedComponent>,
    target: &str,
    diagnostics: &mut Vec<Diagnostic>,
    side: &str,
) -> Option<(String, &'a NormalizedComponent)> {
    if target == "self" {
        return Some((within_component.qualified_name.clone(), within_component));
    }

    let Some(component_use) = within_component.uses.get(target) else {
        diagnostics.push(error(
            "HNK2016",
            format!(
                "Connection `{side}` endpoint references undeclared instance `{target}` within component `{}`.",
                within_component.qualified_name
            ),
        ));
        return None;
    };

    let Some(component_model) = components.get(&component_use.component) else {
        diagnostics.push(error(
            "HNK2017",
            format!(
                "Connection `{side}` endpoint references instance `{target}` whose component `{}` is undefined.",
                component_use.component
            ),
        ));
        return None;
    };

    Some((component_use.component.clone(), component_model))
}

fn resolve_event_name(
    current_package: &str,
    reference: &str,
    event_decls: &BTreeMap<String, EventDecl<'_>>,
    context: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<String> {
    resolve_qualified_name(current_package, reference, event_decls.keys(), "event", context, diagnostics)
}

fn resolve_component_name<T>(
    current_package: &str,
    reference: &str,
    component_decls: &BTreeMap<String, T>,
    context: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<String> {
    resolve_qualified_name(
        current_package,
        reference,
        component_decls.keys(),
        "component",
        context,
        diagnostics,
    )
}

fn resolve_qualified_name<'a>(
    current_package: &str,
    reference: &str,
    known_names: impl Iterator<Item = &'a String>,
    kind: &str,
    context: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<String> {
    let known = known_names.cloned().collect::<Vec<_>>();
    if reference.contains('.') {
        if known.iter().any(|candidate| candidate == reference) {
            return Some(reference.to_string());
        }

        diagnostics.push(error(
            "HNK2005",
            format!("Unknown {kind} reference `{reference}` in {context}."),
        ));
        return None;
    }

    let local_name = qualify(current_package, reference);
    if known.iter().any(|candidate| candidate == &local_name) {
        return Some(local_name);
    }

    let cross_package_matches = known
        .iter()
        .filter(|candidate| candidate.rsplit('.').next() == Some(reference))
        .cloned()
        .collect::<Vec<_>>();

    if !cross_package_matches.is_empty() {
        diagnostics.push(error(
            "HNK2005",
            format!(
                "Unqualified {kind} reference `{reference}` in {context} is ambiguous across packages. Use one of: {}.",
                cross_package_matches.join(", ")
            ),
        ));
        return None;
    }

    diagnostics.push(error(
        "HNK2005",
        format!("Unknown {kind} reference `{reference}` in {context}."),
    ));
    None
}

fn qualify(package: &str, symbol: &str) -> String {
    format!("{package}.{symbol}")
}

fn parse_endpoint(endpoint: &str) -> Option<(&str, &str)> {
    let (target, port) = endpoint.split_once('.')?;
    if target.is_empty() || port.is_empty() {
        return None;
    }
    Some((target, port))
}

fn error(code: &'static str, message: String) -> Diagnostic {
    Diagnostic::new(Severity::Error, code, message, None, None)
}
