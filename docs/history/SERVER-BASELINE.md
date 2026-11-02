# Server baseline reconciliation — 2026-11-02

This records the actual source/deployment evidence used for the first server
task. It does not claim all historical deployments are currently readable/live.

| Source | Retained or current evidence | Limit |
|---|---|---|
| DaemonCraft repository, `f25587eff0db22181ddb6ac289cf4db422d8f792` | Compose selects Purpur 1.21.11, pinned container image, Java/Bedrock and plugin IDs; server-overhaul runbook and client manifest retain richer context | Source configuration is not proof of the last running process or world |
| [Issue 23](https://github.com/nicoechaniz/DaemonCraft/issues/23) | August 28 actual lab protocol/tool smoke; August 29 human conversation/steering acceptance; October 26 deliberate runtime pause | Those are historical acceptance and pause outcomes, not new-stack acceptance |
| Legion deployment | Maintained October pause records identify six stopped containers, preserved data, and disabled project units; new read-only SSH attempt to its Linux route was refused | Current local files/processes could not be reinspected; no Windows or old stack activation was attempted |
| Inference01 deployment | Current authorized SFTP read recovered the Paper unit and backup unit. Paper unit selects an `altercraft` user, a dedicated working directory and start script, with save/stop on shutdown | Candidate directory listing was denied; referenced start script and properties were not available at those paths. Unit existence alone does not prove running version, plugins or current world |
| New ITACA deployment | Fresh native Java/Purpur directory and managed unit, version/hash-locked jars, private bind, server resources and protocol probes | Separate new world; no migrated historical world or human-client acceptance |

The Inference01 SSH credential is command-restricted: arbitrary shell commands
returned no output. The supported SFTP path did provide the service records.
This is a narrower capability, not a claim that the host lacks access entirely.
Detailed private paths, copied service records and receiving observations remain
in the authorized local binding; they are not published here.

## What carries forward

- Game version 1.21.11, peaceful development, flight/commands, normal terrain,
  visible players, skins, permissions, block history, labels and cross-play.
- Rich visuals remain identified as client-side work, avoiding the old drift
  between server features and a modded player client.
- Pin server build/plugin bytes and test actual startup; old CoreProtect 23.1
  metadata did not list 1.21.11, so the new lock selects matching 24.1.
- The new Geyser build's NetherNet defaults differ from the earlier RakNet-only
  deployment. Inspect and configure the pinned build's supported settings.
- Reuse the useful stack behavior without importing the old Hermes bot/cast,
  agent bridge, Redis, backup sidecar or historical machine environment.

Full current Inference01/Legion world/plugin inventories remain gaps. The
automatable new server does not depend on claiming those gaps closed.
