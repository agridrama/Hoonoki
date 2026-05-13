use std::fs;
use std::path::Path;
use std::process::Command;

use hnk_core::{GenerateOptions, generate};
use hnk_idl::load_spec;
use hnk_tools::normalize;
use tempfile::TempDir;

fn pingpong_spec() -> hnk_tools::NormalizedSpec {
    let path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/pingpong/hnk.yaml"
    ));
    let spec = load_spec(path).expect("fixture should parse");
    normalize(&spec).expect("fixture should normalize")
}

#[test]
fn generates_scaffold_and_app_files() {
    let temp = TempDir::new().expect("tempdir should exist");
    let report = generate(&pingpong_spec(), temp.path(), GenerateOptions::default())
        .expect("generation should succeed");

    assert_eq!(report.component_count, 2);
    assert!(temp.path().join("src/generated/mod.rs").exists());
    assert!(temp.path().join("src/generated/runtime_contracts.rs").exists());
    assert!(temp.path().join("src/generated/pinger/wiring.rs").exists());
    assert!(temp.path().join("src/generated/ponger/wiring.rs").exists());
    assert!(temp.path().join("src/app/pinger.rs").exists());
    assert!(temp.path().join("src/app/ponger.rs").exists());
}

#[test]
fn preserves_existing_app_scaffold_and_rewrites_generated_files() {
    let temp = TempDir::new().expect("tempdir should exist");
    let spec = pingpong_spec();

    generate(&spec, temp.path(), GenerateOptions::default()).expect("first generation should succeed");

    let app_file = temp.path().join("src/app/pinger.rs");
    let generated_file = temp.path().join("src/generated/pinger/events.rs");

    fs::write(&app_file, "// preserved custom app\n").expect("app file should be writable");
    fs::write(&generated_file, "// stale generated content\n").expect("generated file should be writable");

    let report = generate(&spec, temp.path(), GenerateOptions::default())
        .expect("second generation should succeed");

    assert!(report.preserved_files.contains(&app_file));
    assert!(report.updated_files.contains(&generated_file));
    assert_eq!(
        fs::read_to_string(&app_file).expect("app file should exist"),
        "// preserved custom app\n"
    );
    assert!(
        !fs::read_to_string(&generated_file)
            .expect("generated file should exist")
            .contains("stale generated content")
    );
}

#[test]
fn generated_wiring_distinguishes_local_and_non_local_dispatch() {
    let temp = TempDir::new().expect("tempdir should exist");
    let spec = pingpong_spec();

    generate(&spec, temp.path(), GenerateOptions::default()).expect("generation should succeed");

    let pinger_wiring =
        fs::read_to_string(temp.path().join("src/generated/pinger/wiring.rs")).unwrap();
    assert!(pinger_wiring.contains("on_non_local_dispatch"));
    assert!(pinger_wiring.contains("dispatch_non_local"));

    let local_spec = r#"
version: "0.1.0"
events:
  - name: Tick
    visibility: internal
    fields: []
  - name: Tock
    visibility: internal
    fields: []
components:
  - name: A
    state:
      fields: []
    ports:
      - name: outbox
        direction: out
        event: Tick
    transitions: []
  - name: B
    state:
      fields: []
    ports:
      - name: inbox
        direction: in
        event: Tick
      - name: outbox
        direction: out
        event: Tock
    transitions:
      - name: on_tick
        on: Tick
        emits: [Tock]
connections:
  - from: A.outbox
    to: B.inbox
actors:
  - name: local
    components: [A, B]
"#;

    let parsed = hnk_idl::parse_spec(None, local_spec).expect("spec should parse");
    let normalized = normalize(&parsed).expect("spec should normalize");
    let local_temp = TempDir::new().expect("tempdir should exist");
    generate(&normalized, local_temp.path(), GenerateOptions::default()).expect("generation should succeed");

    let local_wiring =
        fs::read_to_string(local_temp.path().join("src/generated/a/wiring.rs")).unwrap();
    assert!(local_wiring.contains("dispatch_local"));
}

#[test]
fn generated_hooks_include_must_reply_and_timeout_boundaries() {
    let temp = TempDir::new().expect("tempdir should exist");
    let spec = pingpong_spec();

    generate(&spec, temp.path(), GenerateOptions::default()).expect("generation should succeed");

    let wiring = fs::read_to_string(temp.path().join("src/generated/pinger/wiring.rs")).unwrap();
    assert!(wiring.contains("must_reply_hooks.on_request_started"));
    assert!(wiring.contains("must_reply_hooks.on_reply_missing"));
    assert!(wiring.contains("timeout_hooks.on_timeout_window_started"));
    assert!(wiring.contains("timeout_hooks.on_timeout_contract_violation"));
    assert!(wiring.contains("timeout_hooks.on_timeout_event_fired"));
}

#[test]
fn generated_runtime_contracts_expose_shared_hook_interfaces() {
    let temp = TempDir::new().expect("tempdir should exist");
    let spec = pingpong_spec();

    generate(&spec, temp.path(), GenerateOptions::default()).expect("generation should succeed");

    let runtime_contracts =
        fs::read_to_string(temp.path().join("src/generated/runtime_contracts.rs")).unwrap();
    assert!(runtime_contracts.contains("pub trait RuntimeHooks"));
    assert!(runtime_contracts.contains("pub trait MustReplyHooks"));
    assert!(runtime_contracts.contains("pub trait TimeoutHooks"));
}

#[test]
fn cli_generate_writes_files_to_requested_output_directory() {
    let temp = TempDir::new().expect("tempdir should exist");
    let out_dir = temp.path().join("demo-link");
    let status = Command::new("cargo")
        .arg("run")
        .arg("-q")
        .arg("-p")
        .arg("hnk-core")
        .arg("--")
        .arg("generate")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/link/stubborn_link.yml"
        ))
        .arg("--out")
        .arg(&out_dir)
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .status()
        .expect("cli should run");

    assert!(status.success());
    assert!(out_dir.join("src/generated/stubborn_sender/wiring.rs").exists());
    assert!(out_dir.join("src/app/stubborn_sender.rs").exists());
}
