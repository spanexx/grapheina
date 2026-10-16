#!/usr/bin/env python3
import json
import os
import stat
import subprocess
import tempfile
import unittest
from pathlib import Path

CLI = Path.home() / ".local/bin/agent-notify"


class AgentNotifyTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(prefix="agent-notify-test-")
        self.base = Path(self.temp.name) / "hub"
        self.bin = Path(self.temp.name) / "bin"
        self.bin.mkdir()
        self.hermes_log = Path(self.temp.name) / "hermes.log"
        fake_hermes = self.bin / "hermes"
        fake_hermes.write_text(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \""
            + str(self.hermes_log)
            + "\"\n",
            encoding="utf-8",
        )
        fake_hermes.chmod(fake_hermes.stat().st_mode | stat.S_IXUSR)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def write_fake_speechspark(self) -> None:
        self.speechspark_log = Path(self.temp.name) / "speechspark.log"
        fake = self.bin / "speechspark"
        fake.write_text(
            "#!/usr/bin/env python3\n"
            "import sys\n"
            "args = sys.argv[1:]\n"
            "with open(" + repr(str(self.speechspark_log)) + ", 'a') as log:\n"
            "    log.write(repr(args) + chr(10))\n"
            "path = args[args.index('-o') + 1] if '-o' in args else None\n"
            "if path:\n"
            "    with open(path, 'wb') as f:\n"
            "        f.write(b'FAKEWAV')\n"
            "sys.exit(0)\n",
            encoding="utf-8",
        )
        fake.chmod(fake.stat().st_mode | stat.S_IXUSR)

    def write_fake_ffmpeg(self) -> None:
        fake = self.bin / "ffmpeg"
        fake.write_text(
            "#!/usr/bin/env python3\n"
            "import sys\n"
            "out = sys.argv[-1]\n"
            "with open(out, 'wb') as f:\n"
            "    f.write(b'FAKEOGG')\n"
            "sys.exit(0)\n",
            encoding="utf-8",
        )
        fake.chmod(fake.stat().st_mode | stat.S_IXUSR)

    def write_fake_speechspark_missing_output(self) -> None:
        fake = self.bin / "speechspark"
        fake.write_text("#!/bin/sh\nexit 1\n", encoding="utf-8")
        fake.chmod(fake.stat().st_mode | stat.S_IXUSR)

    def run_cli(self, *args: str) -> subprocess.CompletedProcess[str]:
        env = os.environ.copy()
        env["AGENT_NOTIFY_BASE"] = str(self.base)
        env["PATH"] = f"{self.bin}{os.pathsep}{env['PATH']}"
        return subprocess.run([str(CLI), *args], env=env, text=True, capture_output=True)

    def guestbook(self, event: str) -> list[str]:
        return json.loads((self.base / "events" / f"{event}.guest.json").read_text())

    def test_create_subscribes_and_notifies_victor(self) -> None:
        result = self.run_cli("create", "release-review", "--title", "Review release", "--as", "cline")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.guestbook("release-review"), ["cline", "victor"])
        self.assertIn("Victor subscribed and notified", result.stdout)
        self.assertTrue(self.hermes_log.read_text(encoding="utf-8").strip())

    def test_existing_live_event_is_migrated_without_replay(self) -> None:
        events = self.base / "events"
        events.mkdir(parents=True)
        (events / "old-event.md").write_text("# Old event\n", encoding="utf-8")
        (events / "old-event.guest.json").write_text('["cline"]', encoding="utf-8")
        result = self.run_cli("ls")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.guestbook("old-event"), ["cline", "victor"])
        self.assertFalse(self.hermes_log.exists())

    def test_victor_cannot_unsubscribe(self) -> None:
        created = self.run_cli("create", "protected-event", "--title", "Protected", "--as", "cline")
        self.assertEqual(created.returncode, 0, created.stderr)
        result = self.run_cli("unsub", "protected-event", "--as", "victor")
        self.assertEqual(result.returncode, 2)
        self.assertIn("mandatory subscriber", result.stderr)
        self.assertIn("victor", self.guestbook("protected-event"))

    def test_victor_gets_voice_and_text_on_create(self) -> None:
        self.write_fake_speechspark()
        self.write_fake_ffmpeg()
        result = self.run_cli(
            "create", "release-check-9", "--title", "Release check nine",
            "--ask", "Please approve the release.", "--as", "cline",
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("delivered voice to victor", result.stdout)
        log = self.hermes_log.read_text(encoding="utf-8")
        self.assertIn("[[audio_as_voice]]", log)
        self.assertIn("MEDIA:", log)
        # Text poke still present alongside the voice message.
        self.assertIn("agent-notify: event release-check-9", log)
        spoken = self.speechspark_log.read_text(encoding="utf-8")
        # Event id is spoken (hyphens replaced with spaces), not skipped.
        self.assertIn("release check 9", spoken)
        # No raw ISO timestamp leaks into the spoken text.
        self.assertNotRegex(spoken, r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z")
        self.assertIn("just now", spoken)
        self.assertIn("Please approve the release.", spoken)

    def test_voice_delivery_failure_does_not_block_text(self) -> None:
        self.write_fake_speechspark_missing_output()
        self.write_fake_ffmpeg()
        result = self.run_cli(
            "create", "release-check-10", "--title", "Release check ten", "--as", "cline",
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("voice delivery skipped for victor", result.stdout)
        log = self.hermes_log.read_text(encoding="utf-8")
        self.assertIn("agent-notify: event release-check-10", log)
        self.assertNotIn("[[audio_as_voice]]", log)


if __name__ == "__main__":
    unittest.main()
