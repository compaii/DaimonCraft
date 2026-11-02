# Operations and bounds

For the separate live game server use [SERVER.md](SERVER.md). The Rust harness
currently has an offline fixture. Run its README commands in the foreground;
they make no network calls, start no service and do not access any credentials.

The demo has twelve decision attempts, eight scripted body actions, a 33×33
position extent, thirty-two placed blocks, eight chat lines of at most 256 bytes,
and 4 KiB per event/tool/evidence payload. The core supports explicit budgets of
1–100 attempts. Attention has an explicit capacity; full means the producer
retains the event rather than losing it silently.

The CLI retains one session snapshot capped at 1 MiB plus one temporary file,
replaced after each state transition. A new directory prevents a second writer
or overwrite of old effects. Pending state is synchronized before body execution;
completed evidence is synchronized afterwards, with atomic rename and parent
directory synchronization. This ordering uses standard [Rust file operations](https://doc.rust-lang.org/std/fs/struct.File.html).
This is offline state evidence, not signed Matrix delivery or a resumable live game.

`inspect` reads at most 1 MiB and does not execute anything. A failed write can
leave `session.tmp`; retain it with `session.json` for inspection. Do not rerun an
action to repair a snapshot. The sandbox itself is in-memory; the CLI deliberately
does not resume it as though a snapshot recreated a real world.

Keep at most one completed local demo directory. Inspect and remove that
completed demo before another run; never auto-delete pending/failed evidence.
No automatic run directory creation, recording or background retention job exists.
Stop is ordinary foreground process termination. Future live adapters must supply
actual body cancellation and network deadlines; current synchronous traits do
not establish mid-call interruption or a wall-clock limit for arbitrary adapters.

Builds use the checkout's ignored `target/` on its data filesystem, with debug
symbols and incremental compilation disabled for this small bootstrap. Reuse it
for checks; `cargo clean` removes disposable compiled output when no build uses
it. Retain source, lockfile and unfinished runtime evidence.

The game-server receiving deployment is recorded in its owning server task;
agent game login, model access, authenticated Matrix body registration,
DMK and boot behavior remain unconfigured. Resolve them from current authorized
receiving resources when their implementation is selected. A successful offline
demo does not authorize reactivation of legacy identities/services or model loops.
