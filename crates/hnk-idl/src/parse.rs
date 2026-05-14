use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_yaml::Location;

use crate::ast::Spec;
use crate::diagnostics::{Diagnostic, DiagnosticSet};

#[derive(Debug, Clone, PartialEq)]
pub struct BundleFileAst {
    pub path: PathBuf,
    pub package: String,
    pub imports: Vec<PathBuf>,
    pub spec: Spec,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BundleAst {
    pub entry: PathBuf,
    pub import_root: PathBuf,
    pub files: Vec<BundleFileAst>,
    pub import_graph: BTreeMap<PathBuf, Vec<PathBuf>>,
}

pub fn parse_spec(path: Option<&Path>, source: &str) -> Result<Spec, DiagnosticSet> {
    let spec = serde_yaml::from_str::<Spec>(source).map_err(|error| {
        DiagnosticSet::singleton(classify_parse_error(path, &error.to_string(), error.location()))
    })?;

    validate_package_name(path, &spec.package)?;
    Ok(spec)
}

pub fn load_spec(path: impl AsRef<Path>) -> Result<Spec, DiagnosticSet> {
    let path = path.as_ref();
    let source = fs::read_to_string(path).map_err(|error| {
        DiagnosticSet::singleton(Diagnostic::parse_error(
            "HNK1000",
            Some(path),
            format!("Failed to read the IDL file: {error}"),
            None,
            None,
            Some("Check that the file exists and is readable.".to_string()),
        ))
    })?;

    parse_spec(Some(path), &source)
}

pub fn load_bundle(entry: impl AsRef<Path>, import_root: impl AsRef<Path>) -> Result<BundleAst, DiagnosticSet> {
    let entry = entry.as_ref();
    let import_root = import_root.as_ref();

    let canonical_root = fs::canonicalize(import_root).map_err(|error| {
        DiagnosticSet::singleton(Diagnostic::parse_error(
            "HNK1005",
            Some(import_root),
            format!("Failed to resolve the import root: {error}"),
            None,
            None,
            Some("Pass an existing directory with `--import-root`.".to_string()),
        ))
    })?;

    let canonical_entry = fs::canonicalize(entry).map_err(|error| {
        DiagnosticSet::singleton(Diagnostic::parse_error(
            "HNK1005",
            Some(entry),
            format!("Failed to resolve the entry IDL file: {error}"),
            None,
            None,
            Some("Check that the entry file exists under the chosen import root.".to_string()),
        ))
    })?;

    if !canonical_entry.starts_with(&canonical_root) {
        return Err(DiagnosticSet::singleton(Diagnostic::parse_error(
            "HNK1005",
            Some(entry),
            "The entry file must live under the import root.".to_string(),
            None,
            None,
            Some(format!(
                "Move the entry file under `{}` or choose a broader `--import-root`.",
                canonical_root.display()
            )),
        )));
    }

    let mut state = BundleBuildState::default();
    visit_file(
        &canonical_entry,
        &canonical_root,
        &mut state.visiting,
        &mut state.visited,
        &mut state.files,
        &mut state.import_graph,
    )?;

    Ok(BundleAst {
        entry: canonical_entry,
        import_root: canonical_root,
        files: state.files,
        import_graph: state.import_graph,
    })
}

fn classify_parse_error(
    path: Option<&Path>,
    raw_message: &str,
    location: Option<Location>,
) -> Diagnostic {
    let line = location.as_ref().map(|loc| loc.line());
    let column = location.as_ref().map(|loc| loc.column());

    if let Some(field) = quoted_segment_after(raw_message, "missing field ") {
        return Diagnostic::parse_error(
            "HNK1001",
            path,
            format!(
                "A required field `{field}` is missing from this IDL block."
            ),
            line,
            column,
            Some(format!(
                "Add a `{field}` field to this mapping. If this block is a component, event, or transition, check the Hoonoki spec for the required shape."
            )),
        );
    }

    if let Some(field) = quoted_segment_after(raw_message, "unknown field ") {
        let suggestion = if field == "events" {
            "Ports now accept exactly one event. Replace `events: [MyEvent]` with `event: MyEvent`, and split the port if it previously carried multiple events.".to_string()
        } else {
            let expected = expected_fields(raw_message);
            match expected {
                Some(expected) => format!(
                    "Remove `{field}` or rename it to one of the supported fields: {expected}."
                ),
                None => format!(
                    "Remove `{field}` or rename it to a valid field defined by the Hoonoki IDL spec."
                ),
            }
        };

        return Diagnostic::parse_error(
            "HNK1002",
            path,
            format!(
                "The field `{field}` is not allowed in this part of the IDL."
            ),
            line,
            column,
            Some(suggestion),
        );
    }

    if raw_message.contains("invalid type:") {
        return Diagnostic::parse_error(
            "HNK1003",
            path,
            format!("This value has the wrong YAML type for the Hoonoki IDL: {raw_message}"),
            line,
            column,
            Some(
                "Check whether this field expects a string, list, or mapping, and rewrite the YAML value to match that shape."
                    .to_string(),
            ),
        );
    }

    Diagnostic::parse_error(
        "HNK1000",
        path,
        format!("The IDL could not be parsed as valid Hoonoki YAML: {raw_message}"),
        line,
        column,
        Some(
            "Check indentation, field names, and required fields near this location."
                .to_string(),
        ),
    )
}

fn quoted_segment_after<'a>(message: &'a str, prefix: &str) -> Option<&'a str> {
    let start = message.find(prefix)? + prefix.len();
    let rest = &message[start..];
    let rest = rest.strip_prefix('`')?;
    let end = rest.find('`')?;
    Some(&rest[..end])
}

fn expected_fields(message: &str) -> Option<String> {
    let marker = "expected one of ";
    let start = message.find(marker)? + marker.len();
    let suffix = &message[start..];
    Some(suffix.trim().trim_end_matches('.').to_string())
}

#[derive(Default)]
struct BundleBuildState {
    visiting: Vec<PathBuf>,
    visited: BTreeSet<PathBuf>,
    files: Vec<BundleFileAst>,
    import_graph: BTreeMap<PathBuf, Vec<PathBuf>>,
}

fn visit_file(
    path: &Path,
    import_root: &Path,
    visiting: &mut Vec<PathBuf>,
    visited: &mut BTreeSet<PathBuf>,
    files: &mut Vec<BundleFileAst>,
    import_graph: &mut BTreeMap<PathBuf, Vec<PathBuf>>,
) -> Result<(), DiagnosticSet> {
    if visited.contains(path) {
        return Ok(());
    }

    if let Some(index) = visiting.iter().position(|candidate| candidate == path) {
        let cycle_tail = path.to_path_buf();
        let cycle = visiting[index..]
            .iter()
            .chain(std::iter::once(&cycle_tail))
            .map(|segment| segment.strip_prefix(import_root).unwrap_or(segment).display().to_string())
            .collect::<Vec<_>>()
            .join(" -> ");
        return Err(DiagnosticSet::singleton(Diagnostic::parse_error(
            "HNK1006",
            Some(path),
            format!("Import cycle detected: {cycle}"),
            None,
            None,
            Some("Break the cycle so imports form an acyclic file dependency graph.".to_string()),
        )));
    }

    visiting.push(path.to_path_buf());
    let spec = load_spec(path)?;

    let mut resolved_imports = Vec::with_capacity(spec.imports.len());
    for import in &spec.imports {
        let resolved = resolve_import_path(path, import_root, import)?;
        visit_file(&resolved, import_root, visiting, visited, files, import_graph)?;
        resolved_imports.push(resolved);
    }

    visiting.pop();
    visited.insert(path.to_path_buf());
    import_graph.insert(path.to_path_buf(), resolved_imports.clone());
    files.push(BundleFileAst {
        path: path.to_path_buf(),
        package: spec.package.clone(),
        imports: resolved_imports,
        spec,
    });
    Ok(())
}

fn resolve_import_path(current_file: &Path, import_root: &Path, import: &str) -> Result<PathBuf, DiagnosticSet> {
    let joined = import_root.join(import);
    let canonical = fs::canonicalize(&joined).map_err(|error| {
        DiagnosticSet::singleton(Diagnostic::parse_error(
            "HNK1005",
            Some(current_file),
            format!("Failed to load import `{import}`: {error}"),
            None,
            None,
            Some(format!(
                "Imports must be file paths relative to `{}`.",
                import_root.display()
            )),
        ))
    })?;

    if !canonical.starts_with(import_root) {
        return Err(DiagnosticSet::singleton(Diagnostic::parse_error(
            "HNK1005",
            Some(current_file),
            format!("Import `{import}` escapes the configured import root."),
            None,
            None,
            Some("Keep imported files inside the import root.".to_string()),
        )));
    }

    Ok(canonical)
}

fn validate_package_name(path: Option<&Path>, package: &str) -> Result<(), DiagnosticSet> {
    let is_valid = !package.is_empty()
        && package.split('.').all(is_valid_package_segment);

    if is_valid {
        return Ok(());
    }

    Err(DiagnosticSet::singleton(Diagnostic::parse_error(
        "HNK1004",
        path,
        format!(
            "The package name `{package}` is invalid. Hoonoki packages must use lower_snake segments separated by dots."
        ),
        None,
        None,
        Some("Use names like `std.link` or `examples.broadcast`.".to_string()),
    )))
}

fn is_valid_package_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    matches!(chars.next(), Some('a'..='z'))
        && chars.all(|character| matches!(character, 'a'..='z' | '0'..='9' | '_'))
}
