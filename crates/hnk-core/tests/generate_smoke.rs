use std::fs;
use std::path::Path;
use std::process::Command;

use hnk_core::{GenerateOptions, generate};
use hnk_idl::{load_bundle, load_spec, parse_spec};
use hnk_tools::{normalize, normalize_bundle};
use tempfile::TempDir;

fn stubborn_link_spec() -> hnk_tools::NormalizedSpec {
    let path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/link/stubborn_link.yml"
    ));
    let spec = load_spec(path).expect("fixture should parse");
    normalize(&spec).expect("fixture should normalize")
}

fn repo_root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

#[test]
fn generates_scaffold_and_app_files() {
    let temp = TempDir::new().expect("tempdir should exist");
    let report = generate(&stubborn_link_spec(), temp.path(), GenerateOptions::default())
        .expect("generation should succeed");

    assert_eq!(report.component_count, 3);
    assert!(temp.path().join("src/generated/mod.rs").exists());
    assert!(temp.path().join("src/generated/runtime_contracts.rs").exists());
    assert!(temp.path().join("src/generated/stubborn_link/wiring.rs").exists());
    assert!(temp.path().join("src/generated/fair_loss_link/wiring.rs").exists());
    assert!(temp.path().join("src/app/stubborn_link.rs").exists());
}

#[test]
fn preserves_existing_app_scaffold_and_rewrites_generated_files() {
    let temp = TempDir::new().expect("tempdir should exist");
    let spec = stubborn_link_spec();

    generate(&spec, temp.path(), GenerateOptions::default()).expect("first generation should succeed");

    let app_file = temp.path().join("src/app/stubborn_link.rs");
    let generated_file = temp.path().join("src/generated/stubborn_link/events.rs");

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
    let spec = stubborn_link_spec();

    generate(&spec, temp.path(), GenerateOptions::default()).expect("generation should succeed");

    let wiring =
        fs::read_to_string(temp.path().join("src/generated/stubborn_link/wiring.rs")).unwrap();
    assert!(wiring.contains("on_non_local_dispatch"));
    assert!(wiring.contains("dispatch_non_local"));
    assert!(wiring.contains("dispatch_local"));
    assert!(wiring.contains("\"non_local\""));
    assert!(wiring.contains("\"local\""));
}

#[test]
fn generated_hooks_include_must_reply_and_timeout_boundaries() {
    let source = r#"
version: "0.1.0"
package: examples.inline
imports: []
events:
  - name: Ask
    visibility: public
    version: "1.0.0"
    fields: []
  - name: Reply
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
      - name: reply_out
        direction: out
        event: Reply
    transitions:
      - name: answer
        on: Ask
        emits: [Reply]
  - name: Requester
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
          timeout:
            after: 5s
      - name: reply_in
        direction: in
        event: Reply
    transitions: []
connections:
  - within: Requester
    from: self.request_out
    to: worker.ask_in
    locality: non_local
  - within: Requester
    from: worker.reply_out
    to: self.reply_in
    locality: non_local
"#;

    let parsed = parse_spec(None, source).expect("spec should parse");
    let normalized = normalize(&parsed).expect("spec should normalize");
    let temp = TempDir::new().expect("tempdir should exist");
    generate(&normalized, temp.path(), GenerateOptions::default()).expect("generation should succeed");

    let wiring = fs::read_to_string(temp.path().join("src/generated/requester/wiring.rs")).unwrap();
    assert!(wiring.contains("must_reply_hooks.on_request_started"));
    assert!(wiring.contains("must_reply_hooks.on_reply_missing"));
    assert!(wiring.contains("timeout_hooks.on_timeout_window_started"));
    assert!(wiring.contains("timeout_hooks.on_timeout_contract_violation"));
    assert!(wiring.contains("must_reply_hooks.on_reply_observed"));
}

#[test]
fn generated_runtime_contracts_expose_locality_metadata() {
    let temp = TempDir::new().expect("tempdir should exist");
    let spec = stubborn_link_spec();

    generate(&spec, temp.path(), GenerateOptions::default()).expect("generation should succeed");

    let runtime_contracts =
        fs::read_to_string(temp.path().join("src/generated/runtime_contracts.rs")).unwrap();
    assert!(runtime_contracts.contains("pub trait RuntimeHooks"));
    assert!(runtime_contracts.contains("pub trait MustReplyHooks"));
    assert!(runtime_contracts.contains("pub trait TimeoutHooks"));
    assert!(runtime_contracts.contains("pub locality: &'static str"));
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
    assert!(out_dir.join("src/generated/stubborn_link/wiring.rs").exists());
    assert!(out_dir.join("src/app/stubborn_link.rs").exists());
}

#[test]
fn generate_best_effort_broadcast_fixture() {
    let bundle = load_bundle(
        repo_root().join("examples/broadcast/best_effort_broadcast.yml"),
        repo_root(),
    )
    .expect("bundle should load");
    let normalized = normalize_bundle(&bundle).expect("fixture should normalize");
    let temp = TempDir::new().expect("tempdir should exist");

    let report = generate(&normalized, temp.path(), GenerateOptions::default())
        .expect("generation should succeed");

    assert_eq!(report.component_count, 5);
    assert!(temp.path().join("src/generated/best_effort_broadcast/wiring.rs").exists());
    assert!(temp.path().join("src/generated/perfect_link/wiring.rs").exists());
    assert!(temp.path().join("src/app/best_effort_broadcast.rs").exists());
}
