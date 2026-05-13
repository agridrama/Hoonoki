# Hoonoki Validation Strategy

## Correctness Split

Hoonoki divides correctness into two parts:

1. static validation
2. runtime contracts

This split is intentional. Some guarantees can be checked from structure alone, while others depend on execution behavior.

## Static Validation

MVP static validation is responsible for structural and reference correctness such as:

- undefined events
- undefined components
- undefined ports
- invalid connection direction
- port and event mismatches
- unresolved state field references in `reads` and `writes`
- invalid actor membership references
- duplicate names
- missing `version` on `public` events
- missing declared reply paths for `must_reply`

Compensation flows are validated as ordinary event and transition structure in MVP. There is no special rollback checker yet.

Static validation checks declared possibility and consistency. It does not prove that a specific runtime deployment, participant count, or node topology exists.

It does, however, treat actor boundaries as meaningful. Actor-crossing connections identify where non-local behavior may start, even though the validator still does not bind actors to one exact machine model.

Static validation does not prove full behavioral correctness. It proves that the declared model is internally coherent enough for further tooling.

## Runtime Contracts

Runtime contracts cover guarantees that are difficult or misleading to promise statically in MVP, such as:

- retry behavior
- timeout enforcement
- correlation tracking
- idempotency guards
- actual reply fulfillment for `must_reply`

Timeout modeled as an `event` is different from timeout modeled as a runtime contract. Event-shaped timeouts participate in normal structural validation as part of transitions and connections. Contract-shaped timeouts remain runtime concerns in MVP.

Reusable monitoring or coordination components such as heartbeat monitors and failure detectors are still validated through the same structural rules as any other components. MVP does not give them bespoke validator paths.

These are expected to appear as hook points or generated integration surfaces rather than as a complete runtime implementation.

In practice, runtime contracts matter most when events cross actor boundaries, because those are the places where local call semantics are no longer enough.

## Diagnostic Expectations

Diagnostics should include:

- severity
- stable code
- message
- source path or location
- optional suggestion

## MVP Decisions

- `event.kind` is allowed but not required for validation decisions.
- `requires` and `ensures` are parsed and preserved as annotations.
- `must_reply` is checked statically only for declared reply-path presence. Actual runtime fulfillment is out of scope for static validation.

`must_reply` validation is therefore a possibility check: the model must declare some reply path, but the validator does not prove a concrete deployment-level route or runtime completion.

## Next Diagnostic Improvements

The next stage of parse-diagnostic quality should improve human and LLM readability further.

Planned improvements:

- include a short excerpt of the YAML line or block near the reported location
- suggest likely field-name corrections for misspellings when a close match exists

These improvements are intended to reduce confusion and hallucinated fixes when the IDL is still new and unfamiliar.
