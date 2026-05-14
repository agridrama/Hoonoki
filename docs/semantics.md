# Hoonoki Semantics

## Core Model

Hoonoki models distributed systems as explicit stateful interactions.

- A `component` is a stateful computational unit.
- A `component` may internally use other components as named instances.
- An `event` is a typed interaction.
- A `transition` consumes an input event and may emit further events.
- A `connection` defines how events propagate between component ports.
- A `connection.locality` flag defines whether a propagation step is local or non-local.
- A `contract` describes communication expectations layered on top of the interaction graph.

The model is intentionally about behavioral possibility, not exact machine placement. The IDL says which interactions may happen and which components may participate, without requiring a concrete node-by-node wiring diagram.

## Event Semantics

Events are intentionally broad in MVP.

- An event can be the input that triggers a transition.
- An event can be the observation produced by a transition.
- An event can represent a timeout or timer firing when time itself is part of the modeled behavior.
- The same event type may appear at multiple points in a flow if the model requires it.
- The same component may both consume and emit the same event type when the protocol is symmetric or self-reinforcing.

This keeps the surface language practical while preserving a compositional interpretation underneath.

## Connection Semantics

Connections are read as propagation possibilities.

- A connection says an event may flow from one declared port to another.
- Each declared port carries exactly one event in MVP. Multiple event kinds require multiple ports.
- A connection does not require the IDL to enumerate every runtime participant or machine instance.
- A connection may describe interactions between different component roles or repeated uses of the same component role.

This makes replicated and peer-style protocols expressible without forcing the language to model runtime nodes directly.

Locality refines this interpretation:

- `local` propagation may be direct or in-memory.
- `non_local` propagation may cross a network or another transport boundary.
- Contracts become especially meaningful on `non_local` connections because transport and reply guarantees matter more there.

## Transition Semantics

A transition is the smallest declared unit of behavior.

- `on` identifies the triggering event
- `emits` identifies possible resulting events
- `reads` identifies state dependencies
- `writes` identifies state updates

MVP treats `requires` and `ensures` as preserved annotations. They document intended preconditions and postconditions without yet requiring a full logical checker.

## Compensation Semantics

Compensation is distinct from local transaction rollback.

- Rollback undoes work inside a single transaction boundary.
- Compensation emits valid follow-up behavior that semantically cancels or offsets work that may already have escaped to external systems.

In Hoonoki MVP, compensation is represented with ordinary transitions and events rather than a dedicated language construct. This means saga-style recovery remains expressible without forcing the core DSL to grow a special rollback syntax too early.

## Timeout Semantics

Hoonoki distinguishes two different uses of timeout:

- Timeout as an `event`: a modeled occurrence in the system, such as a lease expiring, a retry window closing, or a saga step timing out.
- Timeout as a `contract`: an operational expectation on an interaction, such as a maximum reply window or request deadline.

When timeout changes the business or protocol state, it should usually appear as an event. When timeout constrains the transport or runtime behavior, it should usually appear as a contract.

## Category-Informed Interpretation

Category theory informs the internal organization of the model, but is not required user-facing syntax.

- Components are treated as object-like units.
- Event-driven transitions are treated as morphism-like steps.
- Connections constrain which compositions are allowed.
- Subsystems are intended to remain composable.

This is a guide for internal semantics and normalization rules, not a commitment to expose mathematical notation in the DSL.

## Why The DSL Stays Practical

Hoonoki does not start from an abstract category DSL. It starts from operational concepts that engineers already need:

- explicit state
- subcomponent composition
- transitions
- ports
- connections
- locality
- contracts

The internal semantics should make those concepts more coherent and analyzable, not less legible.

## Standard Component Direction

Some common distributed patterns are expected to work better as reusable components than as new syntax.

Examples include:

- heartbeat emission
- heartbeat monitoring
- failure detection
- timeout watching
- retry coordination
- lease management
- saga coordination

This preserves a small core language while still leaving room for a higher-level standard library.
