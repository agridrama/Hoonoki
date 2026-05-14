use std::path::Path;

use hnk_idl::{load_bundle, parse_spec};
use hnk_tools::{
    lint, normalize, normalize_bundle, render_inspect_summary, render_mermaid, validate,
    validate_normalized,
};

fn parse(source: &str) -> hnk_idl::Spec {
    parse_spec(None, source).expect("spec should parse")
}

fn bundle_root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

#[test]
fn normalizes_pingpong_and_tracks_non_local_connections() {
    let source = include_str!("../../../examples/pingpong/hnk.yaml");
    let spec = parse(source);
    let normalized = normalize(&spec).expect("normalization should succeed");

    assert_eq!(normalized.events.len(), 4);
    assert_eq!(normalized.components.len(), 3);
    assert_eq!(normalized.non_local_connections.len(), 2);

    let protocol = &normalized.components["examples.pingpong.PingProtocol"];
    assert!(protocol.uses.contains_key("pinger"));
    assert!(protocol.uses.contains_key("ponger"));
}

#[test]
fn validate_accepts_pingpong_fixture() {
    let source = include_str!("../../../examples/pingpong/hnk.yaml");
    let spec = parse(source);
    let diagnostics = validate(&spec);

    assert!(diagnostics.is_empty(), "expected no diagnostics, got {:?}", diagnostics);
}

#[test]
fn validate_rejects_public_event_without_version() {
    let source = r#"
version: "0.1.0"
package: examples.inline
imports: []
events:
  - name: PublicEvent
    visibility: public
    fields: []
components: []
connections: []
"#;

    let diagnostics = validate(&parse(source));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "HNK2101"));
}

#[test]
fn validate_rejects_invalid_state_field_reference() {
    let source = r#"
version: "0.1.0"
package: examples.inline
imports: []
events:
  - name: Tick
    visibility: internal
    fields: []
components:
  - name: Watcher
    state:
      fields: []
    ports:
      - name: timer
        direction: in
        event: Tick
    transitions:
      - name: on_tick
        on: Tick
        reads: [missing]
connections: []
"#;

    let diagnostics = validate(&parse(source));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "HNK2106"));
}

#[test]
fn validate_rejects_missing_reply_path_for_must_reply() {
    let source = r#"
version: "0.1.0"
package: examples.inline
imports: []
events:
  - name: Ask
    visibility: public
    version: "1.0.0"
    fields: []
  - name: Ack
    visibility: internal
    fields: []
components:
  - name: Worker
    state:
      fields: []
    ports:
      - name: ask_in
        direction: in
        event: Ask
    transitions:
      - name: handle
        on: Ask
  - name: RequestFlow
    uses:
      - name: worker
        component: Worker
    state:
      fields: []
    ports:
      - name: request_out
        direction: out
        event: Ask
        contracts:
          must_reply: true
      - name: reply_in
        direction: in
        event: Ack
    transitions: []
connections:
  - within: RequestFlow
    from: self.request_out
    to: worker.ask_in
    locality: non_local
"#;

    let diagnostics = validate(&parse(source));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "HNK2111"));
}

#[test]
fn validate_rejects_connection_direction_mismatch() {
    let source = r#"
version: "0.1.0"
package: examples.inline
imports: []
events:
  - name: Tick
    visibility: internal
    fields: []
components:
  - name: A
    state:
      fields: []
    ports:
      - name: input
        direction: in
        event: Tick
    transitions:
      - name: on_tick
        on: Tick
  - name: Wrapper
    uses:
      - name: a
        component: A
    state:
      fields: []
    ports:
      - name: output
        direction: out
        event: Tick
    transitions: []
connections:
  - within: Wrapper
    from: a.input
    to: self.output
    locality: local
"#;

    let diagnostics = validate(&parse(source));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "HNK2103"));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "HNK2104"));
}

#[test]
fn validate_normalized_keeps_transition_resolution() {
    let source = include_str!("../../../examples/pingpong/hnk.yaml");
    let spec = parse(source);
    let normalized = normalize(&spec).expect("normalization should succeed");

    let diagnostics = validate_normalized(&normalized);
    assert!(diagnostics.is_empty(), "expected no diagnostics, got {:?}", diagnostics);

    let transition = &normalized.components["examples.pingpong.Pinger"].transitions["timeout_ping"];
    assert_eq!(transition.on, "examples.pingpong.PingTimedOut");
    assert_eq!(transition.writes, vec!["outstanding".to_string()]);
}

#[test]
fn lint_warns_on_unused_state_and_unconnected_ports() {
    let source = r#"
version: "0.1.0"
package: examples.inline
imports: []
events:
  - name: Tick
    visibility: internal
    fields: []
components:
  - name: Watcher
    state:
      fields:
        - name: never_used
          type: string
    ports:
      - name: timer
        direction: in
        event: Tick
    transitions:
      - name: on_tick
        on: Tick
connections: []
"#;

    let normalized = normalize(&parse(source)).expect("normalization should succeed");
    let diagnostics = lint(&normalized);

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "HNK2201"));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "HNK2202"));
}

#[test]
fn lint_warns_when_recovery_assumption_has_no_durable_state() {
    let source = r#"
version: "0.1.0"
package: examples.inline
imports: []
events:
  - name: Recover
    visibility: internal
    fields: []
components:
  - name: Store
    assumptions:
      - crash_recovery
    state:
      fields:
        - name: token
          type: string
    ports:
      - name: recover
        direction: in
        event: Recover
    transitions:
      - name: on_recover
        on: Recover
        reads: [token]
connections: []
"#;

    let normalized = normalize(&parse(source)).expect("normalization should succeed");
    let diagnostics = lint(&normalized);

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "HNK2203"));
}

#[test]
fn graph_renders_component_wiring_summary() {
    let source = include_str!("../../../examples/pingpong/hnk.yaml");
    let normalized = normalize(&parse(source)).expect("normalization should succeed");
    let graph = render_mermaid(&normalized);

    assert!(graph.contains("flowchart LR"));
    assert!(graph.contains("component: examples.pingpong.PingProtocol"));
    assert!(graph.contains("non-local: examples.pingpong.PingRequested"));
}

#[test]
fn inspect_summarizes_local_and_non_local_connections() {
    let source = include_str!("../../../examples/link/stubborn_link.yml");
    let normalized = normalize(&parse(source)).expect("normalization should succeed");
    let inspect = render_inspect_summary(&normalized);

    assert!(inspect.contains("non-local: 2"));
    assert!(inspect.contains("uses: fll: examples.link.FairLossLink, timer: examples.link.RetryTimer"));
    assert!(inspect.contains("wiring: 2 local, 2 non-local"));
}

#[test]
fn validate_accepts_best_effort_broadcast_fixture() {
    let bundle = load_bundle(
        bundle_root().join("examples/broadcast/best_effort_broadcast.yml"),
        bundle_root(),
    )
    .expect("bundle should load");
    let normalized = normalize_bundle(&bundle).expect("bundle should normalize");
    let diagnostics = validate_normalized(&normalized);

    assert!(diagnostics.is_empty(), "expected no diagnostics, got {:?}", diagnostics);
}

#[test]
fn inspect_and_graph_show_broadcast_composition() {
    let bundle = load_bundle(
        bundle_root().join("examples/broadcast/best_effort_broadcast.yml"),
        bundle_root(),
    )
    .expect("bundle should load");
    let normalized = normalize_bundle(&bundle).expect("bundle should normalize");

    let inspect = render_inspect_summary(&normalized);
    assert!(inspect.contains("examples.broadcast.BestEffortBroadcast"));
    assert!(inspect.contains("uses: pl: std.link.PerfectLink"));
    assert!(inspect.contains("non-local: 4"));

    let graph = render_mermaid(&normalized);
    assert!(graph.contains("component: examples.broadcast.BestEffortBroadcast"));
    assert!(graph.contains("non-local: std.link.LinkSend"));
}
