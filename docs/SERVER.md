# Development server

The first live task is **server only**. The agent harness remains separate.
The native Java deployment uses no Docker, Hermes, Mineflayer, Redis, agent
bridge or planning service. The Python standard-library utility is an operator
tool, not the Rust cognition runtime.

## Selected baseline

The last retained accepted lab used Purpur 1.21.11. Keep that game/protocol
baseline while the new client is developed. The version, build, exact download
URLs and hashes are in [artifacts.lock.json](../server/artifacts.lock.json):

| Component | Purpose |
|---|---|
| Purpur 1.21.11 build 2568 / Java 21 | Game server; normal terrain, creative development, peaceful, flight and commands |
| LuckPerms | Permissions |
| CoreProtect | Observed block-change history and rollback |
| SkinsRestorer | Skins |
| DecentHolograms | World labels/hologram capability |
| TAB | DaimonCraft header/footer, names and player list |
| Geyser + ViaVersion | Bedrock bridge and protocol translation |
| New DaimonCraft datapack | Coordinates in actionbar, preserved inventory, 2048-block world border |

All seven plugins are enabled in the current server deployment. This does not
prove successful login, rendering or usability from a human game client.
Java status, RCON, plugin state, datapack functions and Bedrock discovery are
separate checks from actual shared play.

Geyser's pinned build supports RakNet and NetherNet. The receiving configuration
uses both, built-in signaling and explicit private advertisement, without STUN
discovery or external signaling registration. Its Java upstream points to the
selected private game bind. Bedrock/RakNet uses UDP 19132; built-in signaling uses
TCP 19132; NetherNet uses UDP 19133. Java defaults to TCP 25565, RCON to TCP 25575.
These are configurable ports, not a public-network exposure instruction.

## Repeatable installation

Prerequisites: Linux, Python 3.11+, curl, Java 21 and systemd for managed operation.
On Debian the runtime package is `openjdk-21-jre-headless`. The server does not
need a GPU. Install the runtime through the host's normal package route.

1. Copy [development.example.json](../server/development.example.json) to a
   receiving-local configuration outside Git. Select a data-filesystem directory,
   specific loopback/VPN bind and admitted player names. The example name is a
   placeholder, not an account to create.
2. Set `DAIMONCRAFT_ROOT` and `DAIMONCRAFT_CONFIG` to the chosen absolute paths.
3. Prepare source-controlled artifacts and game configuration:

```sh
python3 scripts/server.py --root "$DAIMONCRAFT_ROOT" --config "$DAIMONCRAFT_CONFIG" prepare
```

Preparation verifies hashes, downloads only missing pinned artifacts, creates a
private RCON secret once and writes the managed templates. Artifact drift is
reported rather than overwritten. Existing worlds and the secret are preserved.
The directory ownership lock refuses preparation while the server is running.
Configuration application is therefore a deliberate stopped-server operation.

Run directly in the foreground:

```sh
python3 scripts/server.py --root "$DAIMONCRAFT_ROOT" --config "$DAIMONCRAFT_CONFIG" run
```

For managed operation, generate and install the unit through the receiving
administrator. Redirect its output into a local file, then install that file
as `/etc/systemd/system/daimoncraft-server.service`, reload systemd and start
only that unit:

```sh
python3 scripts/server.py --root "$DAIMONCRAFT_ROOT" --config "$DAIMONCRAFT_CONFIG" unit --user "$USER"
```

The generated unit has no boot installation target or automatic restart. It
limits CPU to four cores' worth and memory to the selected heap plus 2 GiB
(default 4 GiB heap / 6 GiB service cap). It can write only its selected state
directory and a private temporary directory. Do not confuse a manually running
server with boot enablement or a registered Matrix body.

## Observe and stop

```sh
python3 scripts/server.py --root "$DAIMONCRAFT_ROOT" --config "$DAIMONCRAFT_CONFIG" status
python3 scripts/server.py --root "$DAIMONCRAFT_ROOT" --config "$DAIMONCRAFT_CONFIG" verify
python3 scripts/server.py --root "$DAIMONCRAFT_ROOT" --config "$DAIMONCRAFT_CONFIG" command --text 'plugins'
python3 scripts/server.py --root "$DAIMONCRAFT_ROOT" --config "$DAIMONCRAFT_CONFIG" command --text 'datapack list enabled'
sudo systemctl stop daimoncraft-server.service
```

`status` speaks the Java server-list protocol, not just a port-open check.
`verify` checks that protocol, all seven enabled plugins, the datapack/objectives,
inventory rule, border and a Bedrock discovery response. Run it after startup
completes; a failure preserves the actual result for diagnosis.
`command` uses authenticated RCON without printing its password. Inspect the
owning unit journal on failure; retain the stopped world and repair forward.
A systemd stop sends the launcher SIGTERM, which requests a normal Java shutdown
and world save, with a 90-second grace before forced termination. Verify the
result before claiming a clean save. Foreground Ctrl-C uses the same route.

Changing config uses stop → prepare → start for this one unit. Do not reactivate
any old identity or old shared service as a deployment dependency.

## Storage, access and upgrades

The default state threshold is 8 GiB with 20 GiB filesystem free reserve. The
launcher checks those before start and every 60 seconds while running, and
requests a graceful stop on breach. These are stop thresholds with bounded
observation delay, not a filesystem quota. World extent, player count, CPU and
memory have explicit limits; there is no world pregeneration or continuous
backup job. Existing ordinary host memory protection is unrelated.

Server logging rolls at 10 MiB with three retained archives. Plugin stores and
bootstrap/remap caches count against the same state threshold. No automatic
world, database or pending-evidence deletion occurs. Before replacing a real
world or changing its format, use the owning consistent-world protection route;
do not recreate historical environment archives or add full-host backup drills.

The initial development mode uses offline game names on a specific private bind
with a whitelist. A public deployment needs its separately selected account and
access configuration. Browser shared login does not substitute for Minecraft
protocol authentication. No account or credential reset is part of this setup.

Upgrades change the lock in a reviewed source change, qualify the changed
artifacts and apply them to a stopped selected server. Existing artifact drift
must be preserved explicitly before replacing files; the installer does not
quietly update a jar or resolve a mutable `latest` reference on restart.

## Visual continuity

The server provides branded MOTD/TAB, skins, hologram support and coordinates.
The earlier rich Java client pack had Sodium/Iris, Complementary Reimagined,
Distant Horizons, Fresh Animations, entity models/textures, connected textures,
leaves, dynamic lights, rain/particles and water wakes. Those are client features;
installing a server cannot turn on a player's shader or Fabric mod.

Their old manifest is retained as design evidence. Rebuilding a matching client
pack is a separate slice after this server-only task. No old client binaries,
caches, world copies, NPC cast or script engine were imported into this stack.
