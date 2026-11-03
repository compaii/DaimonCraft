"""Deployment contract tests, without Java/network access or real credentials."""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

REPO = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("server", REPO / "scripts/server.py")
server = importlib.util.module_from_spec(spec)
spec.loader.exec_module(server)


class DeploymentTests(unittest.TestCase):
    def test_lost_stop_reply_is_not_replayed_or_overridden_by_signal(self):
        class Process:
            def terminate(self):
                raise AssertionError("must not signal after uncertain stop")
        with patch.object(server, "command", side_effect=["saved", ConnectionError("lost reply")]) as call, contextlib.redirect_stdout(io.StringIO()):
            server.shutdown(Path("."), {}, Process())
        self.assertEqual([c.args[2] for c in call.call_args_list], ["save-all flush", "stop"])

    def test_reprepare_retains_world_and_private_secret(self):
        config = server.settings(REPO / "server/development.example.json")
        config["world_border"] = 49152
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "world").mkdir()
            original = b"retained-world-fixture"
            (root / "world/level.dat").write_bytes(original)
            class Defaults:
                def __enter__(self): return self
                def __exit__(self, *_): pass
                def read(self, _): return b"header-footer:\n  enabled: false\nother:\n  value: retained\n"
            with patch.object(server, "artifacts"), patch.object(server, "ZipFile", return_value=Defaults()), contextlib.redirect_stdout(io.StringIO()):
                server.prepare(root, config)
                password = (root / ".rcon-password").read_bytes()
                server.prepare(root, config)
            self.assertEqual((root / ".rcon-password").read_bytes(), password)
            self.assertEqual((root / ".rcon-password").stat().st_mode & 0o777, 0o600)
            self.assertEqual((root / "world/level.dat").read_bytes(), original)
            self.assertIn("max-world-size=24576", (root / "server.properties").read_text())
            self.assertIn("worldborder set 49152", (root / "world/datapacks/daimoncraft/data/daimoncraft/function/load.mcfunction").read_text())
            whitelist = json.loads((root / "whitelist.json").read_text())
            self.assertEqual(whitelist[0]["name"], "Developer")
            self.assertIn("white-list=true", (root / "server.properties").read_text())
            tab = (root / "plugins/TAB/config.yml").read_text()
            self.assertIn("&a&lDaimonCraft", tab)
            self.assertIn("value: retained", tab)

    def test_running_owner_prevents_reconfiguration(self):
        config = server.settings(REPO / "server/development.example.json")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with server.ownership(root), self.assertRaisesRegex(RuntimeError, "owns"):
                server.prepare(root, config)

    def test_public_and_wildcard_binds_are_not_development_defaults(self):
        config = json.loads((REPO / "server/development.example.json").read_text())
        with tempfile.TemporaryDirectory() as directory:
            p = Path(directory) / "config.json"
            for address in ("0.0.0.0", "::", "8.8.8.8"):
                p.write_text(json.dumps(dict(config, bind=address)))
                with self.assertRaises(ValueError):
                    server.settings(p)

    def test_game_varint_boundaries(self):
        class Socket:
            def __init__(self, data):
                self.data = io.BytesIO(data)
            def recv(self, count):
                return self.data.read(count)
        for value in (0, 127, 128, 774, 1048576):
            self.assertEqual(server.read_varint(Socket(server.varint(value))), value)
        with self.assertRaises(RuntimeError):
            server.read_varint(Socket(b"\xff" * 5))


if __name__ == "__main__":
    unittest.main()
