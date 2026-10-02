"""Drive the picker in a real POSIX PTY; verify save/cancel and terminal cleanup."""
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


def pick(config, keys):
    master, slave = pty.openpty()
    before = termios.tcgetattr(slave)
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 26, 90, 0, 0))
    env = dict(os.environ, MOODSH_CONFIG=str(config), TERM="xterm-256color")
    proc = subprocess.Popen([binary, "customize"], stdin=slave, stdout=slave, stderr=slave, env=env)
    output = b""
    sent = False
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
            if not sent and b"Esc cancel" in output:
                os.write(master, keys)
                sent = True
            if proc.poll() is not None:
                break
        assert proc.wait(timeout=1) == 0, output.decode(errors="replace")
        assert sent, output.decode(errors="replace")
        assert before == termios.tcgetattr(slave), "Picker left terminal in raw mode"
        assert b"\x1b[?1049l" in output, "Picker did not leave alternate screen"
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
    print("PASS picker: keyboard navigation, layout toggle, save, cancel, terminal restoration")
