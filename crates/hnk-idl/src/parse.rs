use std::fs;
use std::path::Path;

use serde_yaml::Location;

use crate::ast::Spec;
use crate::diagnostics::{Diagnostic, DiagnosticSet};

pub fn parse_spec(path: Option<&Path>, source: &str) -> Result<Spec, DiagnosticSet> {
    serde_yaml::from_str::<Spec>(source).map_err(|error| {
        DiagnosticSet::singleton(classify_parse_error(path, &error.to_string(), error.location()))
    })
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
