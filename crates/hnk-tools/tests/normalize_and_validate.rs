use hnk_idl::parse_spec;
use hnk_tools::{
    lint, normalize, render_inspect_summary, render_mermaid, validate, validate_normalized,
};

fn parse(source: &str) -> hnk_idl::Spec {
    parse_spec(None, source).expect("spec should parse")
}

#[test]
fn normalizes_pingpong_and_detects_actor_boundary_crossings() {
    let source = include_str!("../../../examples/pingpong/hnk.yaml");
    let spec = parse(source);
    let normalized = normalize(&spec).expect("normalization should succeed");

    assert_eq!(normalized.events.len(), 4);
    assert_eq!(normalized.components.len(), 2);
    assert_eq!(normalized.actor_boundary_crossings.len(), 2);

    let pinger = &normalized.components["Pinger"];
    assert!(pinger.inbound_events.iter().any(|event| event == "PongReturned"));
    assert!(pinger.outbound_events.iter().any(|event| event == "PingRequested"));
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
events:
  - name: PublicEvent
    visibility: public
    fields: []
components: []
connections: []
actors: []
"#;

    let diagnostics = validate(&parse(source));

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "HNK2101"));
}

#[test]
fn validate_rejects_invalid_state_field_reference() {
    let source = r#"
version: "0.1.0"
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
actors:
  - name: node
    components: [Watcher]
"#;

    let diagnostics = validate(&parse(source));

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "HNK2106"));
}

#[test]
fn validate_rejects_missing_reply_path_for_must_reply() {
    let source = r#"
version: "0.1.0"
events:
  - name: Ask
    visibility: public
    version: "1.0.0"
    fields: []
components:
  - name: Requester
    state:
      fields: []
    ports:
      - name: requests
        direction: out
        event: Ask
        contracts:
          must_reply: true
      - name: inbox
        direction: in
        event: Ask
    transitions:
      - name: start
        on: Ask
  - name: Worker
    state:
      fields: []
    ports:
      - name: requests
        direction: in
        event: Ask
    transitions:
      - name: handle
        on: Ask
connections:
  - from: Requester.requests
    to: Worker.requests
actors:
  - name: a
    components: [Requester]
  - name: b
    components: [Worker]
"#;

    let diagnostics = validate(&parse(source));

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "HNK2111"));
}

#[test]
fn validate_rejects_connection_direction_mismatch() {
    let source = r#"
version: "0.1.0"
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
  - name: B
    state:
      fields: []
    ports:
      - name: output
        direction: out
        event: Tick
    transitions: []
connections:
  - from: A.input
    to: B.output
actors: []
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

    let transition = &normalized.components["Pinger"].transitions["timeout_ping"];
    assert_eq!(transition.on, "PingTimedOut");
    assert_eq!(transition.writes, vec!["outstanding".to_string()]);
}

#[test]
fn lint_warns_on_unused_state_and_unconnected_ports() {
    let source = r#"
version: "0.1.0"
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
actors: []
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
actors: []
"#;

    let normalized = normalize(&parse(source)).expect("normalization should succeed");
    let diagnostics = lint(&normalized);

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "HNK2203"));
}

#[test]
fn graph_renders_actor_and_transition_summary() {
    let source = include_str!("../../../examples/pingpong/hnk.yaml");
    let normalized = normalize(&parse(source)).expect("normalization should succeed");
    let graph = render_mermaid(&normalized);

    assert!(graph.contains("flowchart LR"));
    assert!(graph.contains("actor: node-a"));
    assert!(graph.contains("cross-actor: PingRequested"));
    assert!(graph.contains("%% transition adjacency summary"));
}

#[test]
fn inspect_summarizes_local_and_cross_actor_connections() {
    let source = include_str!("../../../examples/pingpong/hnk.yaml");
    let normalized = normalize(&parse(source)).expect("normalization should succeed");
    let inspect = render_inspect_summary(&normalized);

    assert!(inspect.contains("Connections:"));
    assert!(inspect.contains("actor-crossing: 2"));
    assert!(inspect.contains("state-owning component: yes"));
    assert!(inspect.contains("contract-bearing ports: Pinger.requests"));
}
