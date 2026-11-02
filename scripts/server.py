#!/usr/bin/env python3
"""Native Java server deployment; stdlib operator utility, not an agent runtime."""
import argparse
import contextlib
import fcntl
import hashlib
import ipaddress
import json
import os
from pathlib import Path
import re
import secrets
import shutil
import signal
import socket
import struct
import subprocess
import sys
import time
import uuid
from zipfile import ZipFile

REPO = Path(__file__).resolve().parents[1]
LOCK = REPO / "server/artifacts.lock.json"
MAX_ARTIFACT = 128 * 1024 * 1024


def settings(path):
    config = json.loads(path.read_text())
    address = ipaddress.ip_address(config["bind"])
    if not (address.is_private or address.is_loopback) or address.is_unspecified:
        raise ValueError("development bind must be a specific private/loopback address")
    for key in ("port", "rcon_port", "bedrock_port", "bedrock_webrtc_port"):
        if not isinstance(config[key], int) or not 1024 <= config[key] <= 65535:
            raise ValueError(f"invalid {key}")
    if config["port"] == config["rcon_port"]:
        raise ValueError("game and RCON ports must differ")
    for name in config["whitelist"]:
        if not re.fullmatch(r"[A-Za-z0-9_]{1,16}", name):
            raise ValueError("invalid player name")
    if config["online_mode"] and config["whitelist"]:
        raise ValueError("online UUID resolution is not configured; use an empty initial whitelist")
    if not 1024 <= config["heap_mb"] <= 8192:
        raise ValueError("heap_mb outside development bounds")
    if not 1 <= config["data_limit_gib"] <= 32 or config["free_reserve_gib"] < 2:
        raise ValueError("invalid storage bounds")
    return config


def write(path, content, mode=0o600):
    path.parent.mkdir(parents=True, exist_ok=True)
    temp = path.with_name(path.name + ".tmp")
    with temp.open("w") as stream:
        os.chmod(temp, mode)
        stream.write(content)
    temp.replace(path)


@contextlib.contextmanager
def ownership(root):
    root.mkdir(parents=True, exist_ok=True, mode=0o700)
    with (root / ".owner.lock").open("a") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise RuntimeError("server or another preparation owns this directory") from None
        yield


def digest(path, algorithm):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, algorithm).hexdigest()


def artifacts(root):
    manifest = json.loads(LOCK.read_text())
    for artifact in manifest["artifacts"]:
        dest = root / artifact["destination"]
        if not dest.resolve().is_relative_to(root.resolve()):
            raise ValueError("artifact destination escapes root")
        algorithm = "sha512" if "sha512" in artifact else "sha256"
        expected = artifact[algorithm]
        if dest.exists() and digest(dest, algorithm) == expected:
            continue
        if dest.exists():
            raise RuntimeError(f"artifact drift at {artifact['name']}; preserve before replacing")
        dest.parent.mkdir(parents=True, exist_ok=True)
        temp = dest.with_suffix(".download")
        # Exact artifact URLs and hashes live in source. Never resolve 'latest' here.
        subprocess.run(["curl", "--fail", "--location", "--silent", "--show-error",
                        "--max-time", "120", "--max-filesize", str(MAX_ARTIFACT),
                        "--user-agent", "DaimonCraft/0.1 (+https://github.com/compaii/DaimonCraft)",
                        "--output", str(temp), artifact["url"]], check=True)
        if digest(temp, algorithm) != expected:
            raise RuntimeError(f"artifact checksum mismatch: {artifact['name']}")
        temp.replace(dest)
    write(root / "artifacts.lock.json", LOCK.read_text())


def storage_ok(root, config):
    total = sum(p.stat().st_size for p in root.rglob("*") if p.is_file() and not p.is_symlink())
    return (total < config["data_limit_gib"] * 1024**3
            and shutil.disk_usage(root).free >= config["free_reserve_gib"] * 1024**3)


def prepare(root, config):
    with ownership(root):
        if not storage_ok(root, config):
            raise RuntimeError("storage budget/reserve prevents preparation")
        artifacts(root)
        secret = root / ".rcon-password"
        if not secret.exists():
            write(secret, secrets.token_hex(24))
        password = secret.read_text().strip()
        props = {
            "server-ip": config["bind"], "server-port": config["port"],
            "motd": "\\u00a7aDaimonCraft \\u00a78| \\u00a7bExplore, inhabit, create",
            "gamemode": "creative", "difficulty": "peaceful", "pvp": "false",
            "online-mode": str(config["online_mode"]).lower(),
            "white-list": str(bool(config["whitelist"])).lower(),
            "enforce-whitelist": str(bool(config["whitelist"])).lower(),
            "spawn-protection": 0, "allow-flight": "true", "enable-command-block": "true",
            "max-players": 10, "view-distance": 10, "simulation-distance": 6,
            "level-name": "world", "level-seed": config["seed"], "max-world-size": 1024,
            "enable-rcon": "true", "rcon.port": config["rcon_port"], "rcon.password": password,
            "broadcast-rcon-to-ops": "false", "enable-query": "false",
            "sync-chunk-writes": "true", "max-tick-time": 60000,
            "resource-pack": "", "require-resource-pack": "false",
        }
        write(root / "server.properties", "".join(f"{k}={v}\n" for k, v in props.items()))
        write(root / "eula.txt", "eula=true\n")
        # Offline UUID is the protocol's Java nameUUIDFromBytes algorithm.
        players = []
        for name in config["whitelist"]:
            raw = bytearray(hashlib.md5(("OfflinePlayer:" + name).encode()).digest())
            raw[6] = raw[6] & 0x0f | 0x30
            raw[8] = raw[8] & 0x3f | 0x80
            players.append({"uuid": str(uuid.UUID(bytes=bytes(raw))), "name": name})
        write(root / "whitelist.json", json.dumps(players, indent=2))
        templates = REPO / "server/templates"
        for source in templates.rglob("*"):
            if source.is_file():
                content = source.read_text().replace("@BIND@", config["bind"])
                content = content.replace("@BEDROCK_PORT@", str(config["bedrock_port"]))
                content = content.replace("@BEDROCK_WEBRTC_PORT@", str(config["bedrock_webrtc_port"]))
                content = content.replace("@GAME_PORT@", str(config["port"]))
                if source.relative_to(templates).as_posix() == "plugins/TAB/config.yml":
                    with ZipFile(root / "plugins/TAB.jar") as jar:
                        defaults = jar.read("config/config.yml").decode()
                    content = re.sub(r"(?ms)^header-footer:\n.*?(?=^[a-z][\w-]*:\n|\Z)",
                                     lambda _match: content + "\n", defaults, count=1)
                write(root / source.relative_to(templates), content)
        print("Prepared pinned server and seven plugins; existing world retained.")


def exact(sock, count):
    data = b""
    while len(data) < count:
        chunk = sock.recv(count - len(data))
        if not chunk:
            raise ConnectionError("connection closed")
        data += chunk
    return data


def command(root, config, text):
    def packet(sock, kind, body):
        payload = struct.pack("<ii", 1, kind) + body.encode() + b"\0\0"
        sock.sendall(struct.pack("<i", len(payload)) + payload)
        length = struct.unpack("<i", exact(sock, 4))[0]
        if not 10 <= length <= 1024 * 1024:
            raise RuntimeError("invalid RCON response size")
        response = exact(sock, length)
        request_id, response_kind = struct.unpack("<ii", response[:8])
        return request_id, response_kind, response[8:-2].decode(errors="replace")
    with socket.create_connection((config["bind"], config["rcon_port"]), timeout=8) as sock:
        rid, _, _ = packet(sock, 3, (root / ".rcon-password").read_text().strip())
        if rid == -1:
            raise RuntimeError("RCON authentication failed")
        return packet(sock, 2, text)[2]


def varint(number):
    result = b""
    while True:
        byte = number & 127
        number >>= 7
        result += bytes([byte | (128 if number else 0)])
        if not number:
            return result


def read_varint(sock):
    result = 0
    for shift in range(0, 35, 7):
        byte = exact(sock, 1)[0]
        result |= (byte & 127) << shift
        if not byte & 128:
            return result
    raise RuntimeError("invalid VarInt")


def status(config):
    host = config["bind"].encode()
    handshake = b"\0" + varint(774) + varint(len(host)) + host + struct.pack(">H", config["port"]) + b"\1"
    with socket.create_connection((config["bind"], config["port"]), timeout=8) as sock:
        sock.sendall(varint(len(handshake)) + handshake + b"\1\0")
        length = read_varint(sock)
        if length > 1024 * 1024 or read_varint(sock) != 0:
            raise RuntimeError("invalid game status response")
        size = read_varint(sock)
        if size > 1024 * 1024:
            raise RuntimeError("status too large")
        return json.loads(exact(sock, size))


def verify(root, config):
    game = status(config)
    if game["version"]["protocol"] != 774:
        raise RuntimeError("unexpected game protocol")
    plugins = command(root, config, "plugins")
    names = [a["name"] for a in json.loads(LOCK.read_text())["artifacts"] if a["name"] != "Purpur"]
    if not all("\u00a7a" + name in plugins for name in names):
        raise RuntimeError("one or more required plugins are not enabled")
    checks = {
        "datapack": "file/daimoncraft" in command(root, config, "datapack list enabled"),
        "coordinates": all(name in command(root, config, "scoreboard objectives list") for name in ("dc_x", "dc_y", "dc_z")),
        "keep_inventory": "true" in command(root, config, "gamerule keep_inventory"),
        "world_border": "2048" in command(root, config, "worldborder get"),
    }
    magic = bytes.fromhex("00ffff00fefefefefdfdfdfd12345678")
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
        sock.settimeout(5)
        sock.sendto(b"\x01" + struct.pack(">q", int(time.time() * 1000)) + magic + struct.pack(">q", 1),
                    (config["bind"], config["bedrock_port"]))
        data, _ = sock.recvfrom(8192)
        checks["bedrock_discovery"] = bool(data) and data[0] == 0x1c
    if not all(checks.values()):
        raise RuntimeError("server resource verification failed: " + json.dumps(checks))
    return {"game": game["version"], "enabled_plugins": names, **checks,
            "human_gameplay_acceptance": "not established by protocol probes"}


def run(root, config):
    with ownership(root):
        if not storage_ok(root, config):
            raise RuntimeError("storage budget/reserve prevents startup")
        java = subprocess.Popen([config["java"], "-Xms1G", f"-Xmx{config['heap_mb']}M",
                                 "-Dlog4j.configurationFile=log4j2.xml", "-jar", "server.jar", "--nogui"],
                                cwd=root, stdin=subprocess.DEVNULL)
        def stop(_sig=None, _frame=None):
            java.terminate()
        signal.signal(signal.SIGTERM, stop)
        signal.signal(signal.SIGINT, stop)
        next_check = 0
        while java.poll() is None:
            if time.monotonic() >= next_check:
                if not storage_ok(root, config):
                    print("Storage threshold reached; requesting graceful world save and exit.", flush=True)
                    java.terminate()
                    break
                next_check = time.monotonic() + 60
            time.sleep(1)
        try:
            return java.wait(timeout=90)
        except subprocess.TimeoutExpired:
            java.kill()
            java.wait()
            raise RuntimeError("server did not stop within grace; inspect retained world/logs") from None


def unit(root, config_path, user, config):
    for value in (str(root), str(config_path), str(REPO), user):
        if not re.fullmatch(r"[A-Za-z0-9_./-]+", value):
            raise ValueError("unit paths/user must contain plain path characters")
    return f"""[Unit]
Description=DaimonCraft development Minecraft server
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User={user}
WorkingDirectory={root}
ExecStart=/usr/bin/python3 {REPO}/scripts/server.py --root {root} --config {config_path} run
Restart=no
KillSignal=SIGTERM
KillMode=mixed
TimeoutStopSec=100
MemoryMax={config['heap_mb'] + 2048}M
CPUQuota=400%
TasksMax=256
UMask=0077
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths={root}
StandardOutput=journal
StandardError=journal
LogRateLimitIntervalSec=30s
LogRateLimitBurst=100
"""


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--config", type=Path, required=True)
    parser.add_argument("action", choices=["prepare", "run", "status", "verify", "command", "unit"])
    parser.add_argument("--user", default=os.environ.get("USER", "debian"))
    parser.add_argument("--text", default="list")
    args = parser.parse_args()
    config = settings(args.config)
    root = args.root.resolve()
    if args.action == "prepare":
        prepare(root, config)
    elif args.action == "run":
        sys.exit(run(root, config))
    elif args.action == "status":
        print(json.dumps(status(config), indent=2))
    elif args.action == "verify":
        print(json.dumps(verify(root, config), indent=2))
    elif args.action == "command":
        print(command(root, config, args.text))
    else:
        print(unit(root, args.config.resolve(), args.user, config))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, RuntimeError, OSError, subprocess.SubprocessError) as error:
        sys.exit(str(error))
