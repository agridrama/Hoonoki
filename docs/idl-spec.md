# Hoonoki IDL Specification

## Format

The MVP format is YAML. One file describes one spec.

## Top-Level Structure

Required top-level fields:

- `version`
- `events`
- `components`
- `connections`
- `actors`

Optional top-level fields:

- `types`
- `contracts`
- `assumptions`
- `imports`

`imports` is reserved for future use if the project adopts a standard-library style distribution of reusable components and specs.

## Events

Required fields:

- `name`
- `fields`
- `visibility`

Optional fields:

- `version`
- `deprecated`
- `doc`
- `kind`

Notes:

- An event may represent an input that triggers a transition.
- An event may also represent an observable output emitted by a transition.
- An event may represent a timeout or timer firing that drives a transition.
- `kind` is an optional reserved field for future distinctions such as `command`, `fact`, and `signal`.
- MVP validation does not depend on `kind`.
- `public` events are expected to carry `version`.

## Components

Required fields:

- `name`
- `ports`
- `state`
- `transitions`

Optional fields:

- `persistence`
- `assumptions`
- `doc`

Components are explicit stateful units. State is part of the DSL and is not inferred.

In MVP, a component is a behavioral definition, not a concrete runtime node instance.

## Ports

Required fields:

- `name`
- `direction`
- `event`

Optional fields:

- `contracts`
- `doc`

`direction` is one of:

- `in`
- `out`

Each port carries exactly one declared event in MVP.

- Use separate ports when a component needs to handle multiple distinct events.
- Contracts attach to that single-event boundary.

## Transitions

Required fields:

- `name`
- `on`

Optional fields:

- `emits`
- `reads`
- `writes`
- `requires`
- `ensures`
- `doc`

Notes:

- `on` names the triggering input event.
- `emits` lists events that may be observed or forwarded after the transition.
- `reads` and `writes` declare state access.
- `requires` and `ensures` are annotations in MVP. They are parsed and preserved, but not given a full predicate semantics yet.
- The same event type may be consumed and emitted by the same component if the modeled protocol allows it.

## Compensation

The core DSL does not introduce a separate top-level compensation construct in MVP. Compensation is currently modeled as ordinary events and transitions that represent recovery or undo behavior.

Examples of compensation-shaped events:

- `ReserveInventoryCompensated`
- `PaymentAuthorizationCancelled`
- `ShipmentReleaseRequested`

Examples of compensation-shaped transitions:

- a transition triggered by `OrderSagaFailed`
- a transition that emits `CancelPaymentAuthorization`
- a transition that writes recovery state before emitting undo events

This keeps the core DSL small while preserving a path toward future first-class saga helpers if the model proves too repetitive.

## Connections

Required fields:

- `from`
- `to`

Optional fields:

- `contracts`
- `doc`

Connections link `component.port` endpoints and define propagation paths between components.

In MVP, a connection is primarily a declaration that an event can propagate from one component role to another.

- Inside one actor, this may correspond to local dispatch.
- Across actors, this may correspond to networked or otherwise non-local propagation.

The IDL does not require a concrete machine-instance graph, but actor-crossing connections are the main place where contracts matter operationally.

## Actors

Required fields:

- `name`
- `components`

Optional fields:

- `role_selector`
- `routing`
- `doc`

Actors define local execution boundaries for groups of components.

Actors are the main distinction between local interaction and potentially non-local interaction.

- Inside an actor, component interaction may be lightweight and local.
- Across actors, event propagation may require transport guarantees and runtime coordination.

Actors are intentionally lightweight in MVP. They help describe boundaries and analysis scope, but they do not require the IDL to model concrete machine identities one-to-one.

## Contracts

The MVP contract vocabulary includes:

- `delivery`
- `ordering`
- `retry`
- `timeout`
- `must_reply`
- `idempotency`
- `correlation`

Contracts may appear globally or on specific interaction points such as ports or connections.

Contracts are especially important on connections or ports that participate in actor-crossing propagation, because those interactions may rely on non-local transport behavior.

Timeout may be modeled in two distinct ways:

- as an `event` when a timeout is part of the behavioral flow, such as `PaymentTimedOut` or `LeaseExpired`
- as a `contract` when timeout is an interaction guarantee or runtime policy, such as request deadlines or reply windows

These two uses are complementary and should not be treated as duplicates.

## Standard Components

Common distributed-system behaviors should prefer reusable components over dedicated built-in syntax when possible.

Examples that fit this direction:

- heartbeat source or emitter
- heartbeat monitor
- timeout watcher
- failure detector
- retry coordinator
- lease manager
- saga coordinator

In this model, the core DSL stays small and these patterns are expressed with ordinary components, events, transitions, and contracts. Future standard-library distribution may package such components for reuse across specs.
