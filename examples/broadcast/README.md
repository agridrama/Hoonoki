# Best-Effort Broadcast Example

This example models best-effort broadcast as a composite protocol component.

- `BestEffortBroadcast` is the public protocol surface.
- `PerfectLink` is imported from `std/link/perfect_link.hnk.yaml` and used through `uses`.
- The IDL does not enumerate receiver multiplicity or runtime nodes.
- Runtime is expected to expand the `non_local` send path to the appropriate peer set.

The goal of this example is to validate that Hoonoki can describe:

- a composite component with subcomponent instances
- proto-like `package + import` bundle loading
- `self.port` and `instance.port` wiring
- `locality: non_local` propagation that represents transport-sensitive fan-out
