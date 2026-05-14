# Hoonoki Architecture

## Overview

Hoonoki is split into three layers:

1. `hnk-idl`: syntax, parsing, AST, diagnostics
2. `hnk-tools`: normalization, validation, lint, graph, inspect
3. `hnk-core`: code generation and runtime contract hook points

The implementation order is validate-first. Code generation depends on normalized IR and does not define the semantics.

## Responsibility Boundaries

### `hnk-idl`

- Define the YAML syntax
- Parse specs into AST
- Load import bundles from a project root
- Preserve source locations for diagnostics
- Carry syntax-level structures such as `transitions`, `contracts`, `requires`, and `ensures`

### `hnk-tools`

- Resolve references
- Expand defaults and shorthand
- Build `NormalizedSpec`
- Run static validation
- Produce lint warnings and analysis views
- Treat connections as declared propagation possibilities, with special attention to declared locality boundaries

### `hnk-core`

- Consume `NormalizedSpec` only
- Generate Rust scaffolding
- Expose hook points for runtime contracts
- Preserve the `generated` versus `app` ownership boundary

## Design Principles

- Keep the user-facing DSL practical
- Treat category theory as an internal organizing model
- Separate static correctness from runtime guarantees
- Prefer explicit state and transition structure over implicit flow inference
- Prefer reusable standard components over adding narrow built-in syntax for common distributed patterns
- Prefer describing what events can be produced, consumed, and propagated over modeling concrete machine placement in the IDL
- Prefer component composition over deployment-oriented topology in the core DSL
- Prefer proto-like `package + import` separation for reusable specs

## Composition And Locality

A `component` may internally use other components as named instances.

- `uses` declares those subcomponent instances.
- `self.port` refers to the composite component's own boundary.
- `instance.port` refers to a used subcomponent's boundary.
- `connections` describe how events propagate between those endpoints.

`connection.locality` marks whether a propagation step is expected to stay local or cross a non-local runtime boundary.

- `local` means code generation may treat the step as function-call or in-memory dispatch friendly.
- `non_local` means transport, timeout, retry, or observation hooks may be required.

## Current Bootstrap Scope

The current bootstrap workstream defines:

- DSL glossary
- semantics vocabulary
- normalized IR expectations
- validation versus runtime contract boundaries
- the boundary between core DSL and future standard-library components

The current bootstrap workstream already includes the Rust workspace and executable tools, and refinement now focuses on package/import structure, reusable std specs, and example-driven semantics.
