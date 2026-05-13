use std::path::Path;

use hnk_idl::{Spec, load_spec, parse_spec};

fn fixture_path() -> &'static Path {
    Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/pingpong/hnk.yaml"
    ))
}

#[test]
fn parses_pingpong_fixture() {
    let spec = load_spec(fixture_path()).expect("fixture should parse");

    assert_eq!(spec.version, "0.1.0");
    assert_eq!(spec.events.len(), 4);
    assert_eq!(spec.components.len(), 2);

    let pinger = &spec.components[0];
    assert_eq!(pinger.name, "Pinger");
    assert_eq!(pinger.transitions.len(), 3);
    assert_eq!(pinger.transitions[0].on, "StartPing");
    assert_eq!(pinger.transitions[2].on, "PingTimedOut");

    let timeout_event = spec
        .events
        .iter()
        .find(|event| event.name == "PingTimedOut")
        .expect("timeout event should exist");
    assert_eq!(timeout_event.kind.as_deref(), Some("signal"));
}

#[test]
fn parse_from_string_preserves_annotations_and_contracts() {
    let source = r#"
version: "0.1.0"
events:
  - name: TimeoutFired
    visibility: internal
    fields: []
components:
  - name: Watcher
    state:
      fields:
        - name: deadline
          type: string
    ports:
      - name: timers
        direction: in
        event: TimeoutFired
        contracts:
          timeout:
            after: 30s
    transitions:
      - name: observe_timeout
        on: TimeoutFired
        reads: [deadline]
        requires:
          - "deadline is active"
        ensures:
          - "deadline is marked expired"
connections: []
actors:
  - name: node
    components: [Watcher]
"#;

    let spec: Spec = parse_spec(None, source).expect("inline spec should parse");
    let contracts = &spec.components[0].ports[0].contracts;

    assert!(contracts.contains_key("timeout"));
    assert_eq!(spec.components[0].transitions[0].requires.len(), 1);
    assert_eq!(spec.components[0].transitions[0].ensures.len(), 1);
}

#[test]
fn reports_missing_required_field_as_diagnostic() {
    let source = r#"
version: "0.1.0"
events: []
components:
  - name: Broken
    state:
      fields: []
    ports: []
connections: []
actors: []
"#;

    let diagnostics = parse_spec(None, source).expect_err("spec should fail");
    let diagnostic = diagnostics.iter().next().expect("diagnostic should exist");

    assert_eq!(diagnostic.code, "HNK1001");
    assert!(diagnostic.message.contains("required field `transitions` is missing"));
    assert!(
        diagnostic
            .suggestion
            .as_deref()
            .is_some_and(|suggestion| suggestion.contains("Add a `transitions` field"))
    );
}

#[test]
fn reports_unknown_field_as_diagnostic() {
    let source = r#"
version: "0.1.0"
events:
  - name: Foo
    visibility: internal
    fields: []
bogus: true
components: []
connections: []
actors: []
"#;

    let diagnostics = parse_spec(None, source).expect_err("spec should fail");
    let diagnostic = diagnostics.iter().next().expect("diagnostic should exist");

    assert_eq!(diagnostic.code, "HNK1002");
    assert!(diagnostic.message.contains("field `bogus` is not allowed"));
    assert!(
        diagnostic
            .suggestion
            .as_deref()
            .is_some_and(|suggestion| suggestion.contains("Remove `bogus`"))
    );
}

#[test]
fn reports_legacy_port_events_field_with_migration_hint() {
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
        events: [Tick]
    transitions:
      - name: on_tick
        on: Tick
connections: []
actors: []
"#;

    let diagnostics = parse_spec(None, source).expect_err("spec should fail");
    let diagnostic = diagnostics.iter().next().expect("diagnostic should exist");

    assert_eq!(diagnostic.code, "HNK1002");
    assert!(diagnostic.message.contains("field `events` is not allowed"));
    assert!(
        diagnostic
            .suggestion
            .as_deref()
            .is_some_and(|suggestion| suggestion.contains("Replace `events: [MyEvent]` with `event: MyEvent`"))
    );
}

#[test]
fn reports_invalid_type_with_shape_hint() {
    let source = r#"
version: "0.1.0"
events:
  - name: Foo
    visibility: internal
    fields: []
components:
  - name: Broken
    state:
      fields: []
    ports: wrong
    transitions: []
connections: []
actors: []
"#;

    let diagnostics = parse_spec(None, source).expect_err("spec should fail");
    let diagnostic = diagnostics.iter().next().expect("diagnostic should exist");

    assert_eq!(diagnostic.code, "HNK1003");
    assert!(diagnostic.message.contains("wrong YAML type"));
    assert!(
        diagnostic
            .suggestion
            .as_deref()
            .is_some_and(|suggestion| suggestion.contains("expects a string, list, or mapping"))
    );
}
