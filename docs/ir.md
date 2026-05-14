# Hoonoki Normalized IR

## Purpose

The normalized IR is the contract between parsing and downstream tooling.

- `hnk-idl` produces syntax-level AST
- `hnk-tools` produces resolved normalized IR
- `hnk-core` consumes normalized IR only

## Normalization Goals

`NormalizedSpec` must remove ambiguity that would otherwise force every downstream consumer to reinterpret the AST.

The normalized IR is expected to provide:

- resolved references
- fully qualified names
- source file and package provenance
- applied defaults
- explicit visibility and version metadata
- resolved contracts
- resolved subcomponent uses
- per-component inbound and outbound event views
- local and non-local connection views
- import graph metadata
- resolved transition input, outputs, reads, and writes

The normalized IR captures declared interaction possibility, not concrete runtime instantiation. In particular, connections remain resolved propagation relationships between `self.port` and `instance.port` endpoints rather than runtime node edges.

## Expected Nodes

The MVP normalized model includes:

- `NormalizedSpec`
- `NormalizedEvent`
- `NormalizedComponent`
- `NormalizedPort`
- `NormalizedTransition`
- `NormalizedConnection`
- `NormalizedComponentUse`
- `NormalizedStateField`
- `NormalizedContractSet`

## Semantics Preservation

Normalization should preserve the meaning of the source spec while making it easier to validate and generate from.

Specific expectations:

- `event.kind` may be preserved but must not become a mandatory semantics branch in MVP.
- `requires` and `ensures` must be preserved as annotations.
- `must_reply` may become a resolved contract entry, but runtime fulfillment is not proven at this layer.

## Downstream Constraint

`hnk-core` must not re-parse or reinterpret the AST. If code generation needs additional structure, that structure belongs in normalized IR.
