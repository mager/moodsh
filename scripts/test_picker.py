"""Drive the picker in a real POSIX PTY; verify preview, save/cancel and cleanup."""
import errno
import fcntl
import os
from pathlib import Path
import pty
import select
import struct
import subprocess
import sys
import tempfile
import termios
import time
import tomllib

binary = str(Path(sys.argv[1]).resolve())


def pick(config, actions, size=(24, 80), colors=True):
    if isinstance(actions, bytes):
        actions = [(b"Esc cancel", actions)]
    master, slave = pty.openpty()
    before = termios.tcgetattr(slave)
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", *size, 0, 0))
    env = dict(os.environ, MOODSH_CONFIG=str(config), TERM="xterm-256color")
    env.pop("NO_COLOR", None)
    if not colors:
        env["NO_COLOR"] = "1"
    proc = subprocess.Popen([binary, "customize"], stdin=slave, stdout=slave, stderr=slave, env=env)
    output = b""
    stage = 0
    start = 0
    deadline = time.monotonic() + 10
    try:
        while time.monotonic() < deadline:
            if select.select([master], [], [], 0.1)[0]:
                try:
                    output += os.read(master, 65536)
                except OSError as error:
                    if error.errno == errno.EIO:
                        break
                    raise
            if stage < len(actions) and actions[stage][0] in output[start:]:
                os.write(master, actions[stage][1])
                start = len(output)
                stage += 1
            if proc.poll() is not None:
                # Drain terminal restoration output even if process exit won the race.
                while select.select([master], [], [], 0)[0]:
                    output += os.read(master, 65536)
                break
        assert proc.wait(timeout=1) == 0, output.decode(errors="replace")
        assert stage == len(actions), output.decode(errors="replace")
        assert before == termios.tcgetattr(slave), "Picker left terminal in raw mode"
        assert b"\x1b[?1049l" in output, "Picker did not leave alternate screen"
        assert b"\x1b[?2004l" in output, "Picker did not disable bracketed paste"
        return output
    finally:
        if proc.poll() is None:
            proc.kill()
            proc.wait()
        os.close(master)
        os.close(slave)


with tempfile.TemporaryDirectory() as root:
    config = Path(root) / "config.toml"
    pick(config, b"j\t\r")
    saved = config.read_text()
    mood = tomllib.loads(saved)["mood"]
    assert mood["name"] == "aurora", mood
    assert mood["layout"] == "two-line", mood
    pick(config, b"jq")
    assert config.read_text() == saved, "Cancel changed the saved config"

    # See the actual ANSI preview before applying the color, then save the draft.
    output = pick(config, [
        (b"Esc cancel", b"1#12abEF"),
        (b"\x1b[38;2;18;171;239m", b"\r"),
        (b"Esc cancel", b"\r"),
    ])
    mood = tomllib.loads(config.read_text())["mood"]
    assert mood["palette"]["accent"] == "#12abEF", mood
    assert mood["palette"]["path"] == "#89DCEB", mood
    assert mood["layout"] == "two-line", mood
    saved = config.read_text()

    # Invalid input cannot be applied. Discarding an edit keeps the old color.
    pick(config, [
        (b"Esc cancel", b"2#12\r"),
        (b"Enter 6 hex digits", b"\x1b"),
        (b"Esc cancel", b"\r"),
    ])
    assert config.read_text() == saved, "Invalid or discarded edit changed the config"

    # Bracketed paste accepts a copied color with a trailing newline, without saving.
    pick(config, [
        (b"Esc cancel", b"4\x1b[200~#112233\n\x1b[201~"),
        (b"\x1b[38;2;17;34;51m", b"\r"),
        (b"Esc cancel", b"\x1b"),
    ])
    assert config.read_text() == saved, "Cancel wrote an applied draft"
    pick(config, b"1FFFFFF\x03")
    assert config.read_text() == saved, "Ctrl-C wrote an unfinished edit"

    output = pick(config, b"q", colors=False)
    assert b"Color preview disabled" in output
    assert b"\x1b[38;2;" not in output
    pick(config, [(b"Resize to 80 x 24", b"\r\x1b")], size=(12, 40))
    assert config.read_text() == saved, "Small terminal accepted an unseen save"
    absent = Path(root) / "absent.toml"
    pick(absent, b"1FFFFFF\r\x1b")
    assert not absent.exists(), "Cancel created a new config"
    subprocess.run([binary, "path", "--color", "terminal", "--format", "full"], env=dict(os.environ, MOODSH_CONFIG=str(config)), check=True, capture_output=True)
    output = pick(config, b"j\r")
    assert b"\x1b[39m/code/moodsh" in output
    assert tomllib.loads(config.read_text())["path"] == {"color": "terminal", "format": "full"}
    saved = config.read_text()
    pick(config, b"pq")
    assert config.read_text() == saved, "Cancelled path toggle changed config"
    pick(config, b"p\r")
    assert tomllib.loads(config.read_text())["path"]["color"] == "theme"
    print("PASS picker: live colors, paste, validation, save/cancel, small terminals, cleanup")
