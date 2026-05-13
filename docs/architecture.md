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
- Preserve source locations for diagnostics
- Carry syntax-level structures such as `transitions`, `contracts`, `requires`, and `ensures`

### `hnk-tools`

- Resolve references
- Expand defaults and shorthand
- Build `NormalizedSpec`
- Run static validation
- Produce lint warnings and analysis views
- Treat connections as declared propagation possibilities, with special attention to actor-boundary crossings

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

## Actor Boundary

An `actor` is a local execution boundary for a group of components.

- Components inside one actor are assumed to coordinate locally.
- Event propagation inside an actor may be implemented as function calls, in-memory dispatch, or other local mechanisms.
- Event propagation across actors may cross a network or other non-local transport boundary.

This is why connection contracts become especially important at actor crossings. The IDL still does not require `actor` to mean one specific physical machine or process model, but actor boundaries are intended to mark where non-local communication concerns begin.

## Current Bootstrap Scope

The current bootstrap workstream defines:

- DSL glossary
- semantics vocabulary
- normalized IR expectations
- validation versus runtime contract boundaries
- the boundary between core DSL and future standard-library components

The current bootstrap workstream does not yet create the Rust workspace or executable tools.
