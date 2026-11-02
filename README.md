# DaimonCraft

A new Rust project for exploring embodied existence in a shared 3D world.
Minecraft is the first concrete experiment for a small, reusable daimon harness.

This starts the software stack from zero. [DaemonCraft](https://github.com/nicoechaniz/DaemonCraft)
is a source of ideas, experience and historical evidence, not this repository's
runtime or Git ancestry. No Hermes, Python agent loop, Mineflayer service or old
deployment is imported.

## Working now

- [Automatable development server](docs/SERVER.md): native Purpur 1.21.11,
  seven pinned plugins, private Java/Bedrock listeners, branded presentation,
  coordinates and explicit resource/storage bounds. The live server is separate
  from the offline Rust fixture and does not start an agent.
- `daimon-harness`: domain-independent model/body interfaces, bounded working
  state, prioritized attention and correlated action evidence.
- `daimon-minecraft`: a tiny deterministic voxel sandbox for exercising that
  boundary. It is not a Minecraft protocol client.
- `daimoncraft`: an offline scripted perceive → move → observe → chat → place →
  observe → remove → observe demonstration, with atomic state snapshots.
- Recovery retains uncertain pending actions rather than sending them again.

## Run

Install Rust 1.95.0, then:

```sh
cargo test --workspace --locked
mkdir -p .runs
cargo run --locked -p daimoncraft -- demo .runs/first
cargo run --locked -p daimoncraft -- inspect .runs/first/session.json
```

`demo` requires a new directory and refuses to overwrite a previous run.
The model is scripted and the world is simulated: there is no model charge,
live player, game login or Matrix enrollment in this demo.
Delete a completed `.runs/first` after inspecting it; preserve any run containing
unresolved pending state. See [operations](docs/OPERATIONS.md) for bounds.

## Design and continuity

- [Architecture](docs/design/ARCHITECTURE.md): small Rust core, replaceable adapters.
- [Cognition](docs/design/COGNITION.md): consciousness, attention, memory,
  interoception, reflection and learning, with historical sources.
- [Roadmap](docs/ROADMAP.md): runnable slices toward shared play and a general harness.
- [Historical roadmap](docs/history/ROADMAP.md): recovered ideas and later corrections.
- [Source register](docs/history/SOURCES.md): provenance and explicit gaps.
- [Harness² and APN](docs/references/HARNESS-RESEARCH.md): grounded reuse and unresolved references.

The intended Minecraft incarnation is another authenticated Matrix body of
CompAII. A prompt, model, executable or avatar cannot establish identity.
Matrix/Cluster admission and DMK continuity remain integrations to implement.

The general harness may eventually travel as an executable, versioned capability
with the daimon's inherited material (the proposed “ADN” idea). This is a design
direction, not an already adopted species release, identity change or guarantee
that one binary runs on every future body.
