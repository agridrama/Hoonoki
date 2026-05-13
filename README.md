# Hoonoki

Hoonoki is a semantics-first IDL project for distributed systems.

The DSL stays operational and practical on the surface:

- `events`
- `components`
- `transitions`
- `connections`
- `actors`
- `contracts`

The underlying model is compositional and stateful. Category theory informs the internal semantics, but it is not exposed as required end-user syntax.

## Current Status

This repository is in bootstrap mode. The current approved workstream defines the semantics and documentation structure before Rust workspace implementation begins.

## Specification Sources

The primary specifications live under `docs/`:

- `docs/architecture.md`
- `docs/idl-spec.md`
- `docs/semantics.md`
- `docs/ir.md`
- `docs/validation.md`

Read `docs/` before implementing AST, normalization, validation, linting, or code generation.
