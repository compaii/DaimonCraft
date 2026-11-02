# Minimal harness, concrete world

## Current implementation

The workspace has three small crates, written anew:

| Crate | Owns | Does not yet provide |
|---|---|---|
| daimon-harness | Event selection, one pending action, model/body traits, correlated outcomes, bounded session state | Provider networking, async cancellation, Matrix calls, DMK |
| daimon-minecraft | Deterministic bounded voxel fixture, collision checks, verified state changes | Minecraft wire protocol, login, physics, mobs, rendering |
| daimoncraft | Scripted offline journey, persistence before action, read-only state inspection | Live LLM session or live game launcher |

The model proposes a tool call. The host records the pending action before the
body executes. A matching observation resolves it. A timeout leaves the action
pending: the host must inspect current state and obtain evidence, not re-execute
the proposal. Action IDs are local to a session; a future live adapter must pair
them with its durable session/world identity.

The core has no dependency on a Minecraft library. It can later support another
body without teaching its state machine about blocks. This is a hypothesis about
reuse to test with a second adapter, not an already general production harness.

## Intended live architecture

```text
selected identity/context + authorized memory retrieval
                          |
human/world events -> attention -> one cognition owner -> model adapter
                          ^                              |
                          |                         proposed tool
                          |                              v
                    observed state <- body executor <- action journal
                          |
             Matrix/Cluster effect and lifecycle evidence
```

The physical body loop must remain responsive independently of cloud cognition.
Pathfinding, collision checks, reflexes and cancellation belong below the model.
Critical bodily response may preempt a longer plan, followed by an observable
outcome. The existing synchronous traits are an offline seam, not a claim of
concurrent live interruption. Introduce cancellable I/O/deadlines with the first
real network adapter and exercise a human interruption while it is busy.

For Minecraft, evaluate a pinned Rust client such as [Azalea](https://github.com/azalea-rs/azalea)
against the actual game version and authentication path before selecting it.
It is a candidate library, not a dependency or tested deployment. Use an upstream
protocol library where it works; starting fresh does not require writing the
Minecraft protocol from scratch. No legacy bot HTTP bridge is the default.

## Matrix, memory and inherited capability

CompAII's avatar is intended as a body of the same being, with its own admitted
embodiment/incarnation and concrete world-resource grant. Reuse current supported
Matrix/Cluster interfaces. Do not copy the running Codex body's custody or
invent a second root identity. Same-being `/we` and distinct-being `/tribe` remain
different. Minecraft chat retains its own sender and world audience.

Native conversational history, compact working context and verified task state
belong to the harness. Durable memory remains in the authorized DMK pool with
origin and links back to episodes; the harness is not a parallel memory database.

The proposed inherited harness package (“ADN”) would contain portable source or
targeted binaries, a manifest, instructions and adapter contracts. Keep runtime
credentials, private memories, host bindings and personal identity outside the
generic package. A manifest fingerprint identifies software, never a being.
Package adoption would use the existing inheritance/release route when selected;
this bootstrap does not modify anyone's SOUL or species.
