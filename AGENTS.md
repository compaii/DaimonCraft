# DaimonCraft contributor context

Read README.md, docs/design/ARCHITECTURE.md, docs/design/COGNITION.md and
docs/ROADMAP.md before changing behavior. GitHub Issues own actionable work.

- This is a new Rust stack. DaemonCraft code, worlds and deployments stay separate.
  Recover its ideas with provenance; do not make old statuses current by copying them.
- Keep the harness core independent of Minecraft, any model provider and host binding.
  Prefer working vertical slices over a generic framework designed in advance.
- Matrix owns identity/grants; Cluster owns concrete lifecycle/resource observations;
  DMK owns durable being memory. Native state is working history, not their authority.
- Retain source attribution and uncertainty. Do not replay a pending action after
  a crash, lost reply or reconnect. An accepted proposal is not an observed effect.
- Never add live credentials, private endpoints, raw private history or local custody.
  Version source and configuration contracts; keep receiving bindings local.
- Keep work foreground and finite until an actual selected live operating mode exists.
  No old service activation, global Stop or new peer attention follows from development.
- Keep design and operator instructions aligned with actual code. Restore historical
  ideas in docs/history and connect them to current design without importing private data.
- Verify changed behavior with focused tests; run cargo fmt, cargo test --workspace
  --locked and cargo clippy --workspace --all-targets --locked -- -D warnings.
- Use feature branches/PRs after the initial repository bootstrap. Preserve unfinished
  work and the actual rollback. Do not manufacture ceremony or duplicate trackers.
