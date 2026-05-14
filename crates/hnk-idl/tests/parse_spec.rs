use std::fs;
use std::path::Path;

use hnk_idl::{Spec, load_bundle, load_spec, parse_spec};
use tempfile::TempDir;

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
    assert_eq!(spec.package, "examples.pingpong");
    assert_eq!(spec.events.len(), 4);
    assert_eq!(spec.components.len(), 3);
    assert_eq!(spec.connections.len(), 2);
    assert!(spec.components.iter().any(|component| component.name == "PingProtocol"));

    let timeout_event = spec
        .events
        .iter()
        .find(|event| event.name == "PingTimedOut")
        .expect("timeout event should exist");
    assert_eq!(timeout_event.kind.as_deref(), Some("signal"));
}

#[test]
fn parse_from_string_preserves_uses_and_locality() {
    let source = r#"
version: "0.1.0"
package: examples.inline
imports: []
events:
  - name: TimeoutFired
    visibility: internal
    fields: []
components:
  - name: Timer
    state:
      fields: []
    ports:
      - name: timeout_out
        direction: out
        event: TimeoutFired
    transitions: []
  - name: Watcher
    uses:
      - name: timer
        component: Timer
    state:
      fields:
        - name: deadline
          type: string
    ports:
      - name: timeout_in
        direction: in
        event: TimeoutFired
    transitions:
      - name: observe_timeout
        on: TimeoutFired
        reads: [deadline]
connections:
  - within: Watcher
    from: timer.timeout_out
    to: self.timeout_in
    locality: local
"#;

    let spec: Spec = parse_spec(None, source).expect("inline spec should parse");
    assert_eq!(spec.components[1].uses[0].name, "timer");
    assert_eq!(spec.connections[0].within, "Watcher");
}

#[test]
fn reports_missing_required_field_as_diagnostic() {
    let source = r#"
version: "0.1.0"
package: examples.inline
imports: []
events: []
components:
  - name: Broken
    state:
      fields: []
    ports: []
connections: []
"#;

    let diagnostics = parse_spec(None, source).expect_err("spec should fail");
    let diagnostic = diagnostics.iter().next().expect("diagnostic should exist");

    assert_eq!(diagnostic.code, "HNK1001");
    assert!(diagnostic.message.contains("required field `transitions` is missing"));
}

#[test]
fn reports_legacy_port_events_field_with_migration_hint() {
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
        events: [Tick]
    transitions:
      - name: on_tick
        on: Tick
connections: []
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
fn reports_legacy_actor_field_as_unknown() {
    let source = r#"
version: "0.1.0"
package: examples.inline
imports: []
events: []
components: []
connections: []
actors: []
"#;

    let diagnostics = parse_spec(None, source).expect_err("spec should fail");
    let diagnostic = diagnostics.iter().next().expect("diagnostic should exist");
    assert_eq!(diagnostic.code, "HNK1002");
    assert!(diagnostic.message.contains("field `actors` is not allowed"));
}

#[test]
fn reports_invalid_type_with_shape_hint() {
    let source = r#"
version: "0.1.0"
package: examples.inline
imports: []
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
"#;

    let diagnostics = parse_spec(None, source).expect_err("spec should fail");
    let diagnostic = diagnostics.iter().next().expect("diagnostic should exist");

    assert_eq!(diagnostic.code, "HNK1003");
    assert!(diagnostic.message.contains("wrong YAML type"));
}

#[test]
fn rejects_invalid_package_name() {
    let source = r#"
version: "0.1.0"
package: Bad-Package
imports: []
events: []
components: []
connections: []
"#;

    let diagnostics = parse_spec(None, source).expect_err("spec should fail");
    let diagnostic = diagnostics.iter().next().expect("diagnostic should exist");
    assert_eq!(diagnostic.code, "HNK1004");
    assert!(diagnostic.message.contains("package name `Bad-Package` is invalid"));
}

#[test]
fn loads_broadcast_bundle_with_transitive_std_imports() {
    let entry = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/broadcast/best_effort_broadcast.yml"
    ));
    let import_root = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));

    let bundle = load_bundle(entry, import_root).expect("bundle should load");

    assert_eq!(bundle.files.len(), 5);
    assert!(bundle.files.iter().any(|file| file.package == "examples.broadcast"));
    assert!(bundle.files.iter().any(|file| file.package == "std.link"));
    assert!(bundle.files.iter().any(|file| file.package == "std.time"));
}

#[test]
fn reports_missing_import_file() {
    let temp = TempDir::new().expect("tempdir should exist");
    let entry = temp.path().join("entry.hnk.yaml");
    fs::write(
        &entry,
        r#"
version: "0.1.0"
package: examples.temp
imports:
  - std/missing.hnk.yaml
events: []
components: []
connections: []
"#,
    )
    .expect("fixture should write");

    let diagnostics = load_bundle(&entry, temp.path()).expect_err("bundle should fail");
    let diagnostic = diagnostics.iter().next().expect("diagnostic should exist");
    assert_eq!(diagnostic.code, "HNK1005");
    assert!(diagnostic.message.contains("Failed to load import `std/missing.hnk.yaml`"));
}

#[test]
fn reports_import_cycle() {
    let temp = TempDir::new().expect("tempdir should exist");
    let a = temp.path().join("a.hnk.yaml");
    let b = temp.path().join("b.hnk.yaml");
    fs::write(
        &a,
        r#"
version: "0.1.0"
package: examples.cycle
imports:
  - b.hnk.yaml
events: []
components: []
connections: []
"#,
    )
    .expect("fixture should write");
    fs::write(
        &b,
        r#"
version: "0.1.0"
package: examples.cycle
imports:
  - a.hnk.yaml
events: []
components: []
connections: []
"#,
    )
    .expect("fixture should write");

    let diagnostics = load_bundle(&a, temp.path()).expect_err("bundle should fail");
    let diagnostic = diagnostics.iter().next().expect("diagnostic should exist");
    assert_eq!(diagnostic.code, "HNK1006");
    assert!(diagnostic.message.contains("Import cycle detected"));
}
