"""Exercise Zsh's actual line editor in a controlling PTY, using only fixture history."""
import errno
import fcntl
import os
from pathlib import Path
import pty
import select
import shlex
import shutil
import signal
import struct
import subprocess
import sys
import tempfile
import termios
import time

binary = str(Path(sys.argv[1]).resolve())
zsh = shutil.which('zsh')
if not zsh:
    raise SystemExit('Zsh is required for the suggestion tests')


class Session:
    def __init__(self, root, history, flags=(), before='', no_color=False, term='xterm-256color'):
        self.root = root
        self.capture = root / 'capture'
        self.capture.unlink(missing_ok=True)
        self.output = b''
        env = dict(os.environ, HOME=str(root), ZDOTDIR=str(root),
                   MOODSH_CONFIG=str(root / 'config.toml'), TERM=term,
                   CAPTURE=str(self.capture))
        env['PATH'] = str(Path(binary).parent) + os.pathsep + env.get('PATH', '')
        env.pop('NO_COLOR', None)
        if no_color:
            env['NO_COLOR'] = '1'
        init = root / 'init.zsh'
        init.write_bytes(subprocess.check_output([binary, 'init', 'zsh', *flags], env=env))
        (root / 'history').write_text('\n'.join(history) + '\n')
        (root / '.zsh_history').touch()
        (root / '.zshrc').touch()
        startup = root / 'startup.zsh'
        startup.write_text(r'''
HISTFILE=; HISTSIZE=200; SAVEHIST=0
bindkey -e
_moodsh_test_capture() {
  builtin print -rn -- "$BUFFER"$'\0'"$POSTDISPLAY"$'\0'"${(j: :)region_highlight}"$'\0'"$CURSOR" > "$CAPTURE"
}
zle -N _moodsh_test_capture
bindkey '^X^B' _moodsh_test_capture
autoload -Uz add-zle-hook-widget
add-zle-hook-widget line-pre-redraw _moodsh_test_capture
add-zle-hook-widget line-init _moodsh_test_capture
''' + before + f'''
source {shlex.quote(str(init))}
source {shlex.quote(str(init))}
fc -p
HISTSIZE=200
fc -R {shlex.quote(str(root / 'history'))}
_moodsh_test_ready() {{ PROMPT='MOODSH_TEST> '; RPROMPT=; }}
precmd_functions+=(_moodsh_test_ready)
''')
        self.pid, self.fd = pty.fork()
        if self.pid == 0:
            os.chdir(root)
            os.execve(zsh, [zsh, '-d', '-f', '-i'], env)
        fcntl.ioctl(self.fd, termios.TIOCSWINSZ, struct.pack('HHHH', 30, 100, 0, 0))
        self.send(('source ' + shlex.quote(str(startup)) + '\n').encode())
        deadline = time.monotonic() + 10
        while b'MOODSH_TEST> ' not in self.output:
            self.drain()
            if time.monotonic() >= deadline:
                self.close()
                raise AssertionError(self.output.decode(errors='replace'))

    def send(self, data):
        os.write(self.fd, data)

    def drain(self):
        if select.select([self.fd], [], [], 0.05)[0]:
            try:
                self.output += os.read(self.fd, 65536)
            except OSError as error:
                if error.errno != errno.EIO:
                    raise

    def state(self, buffer, suggestion, timeout=5, cursor=None, highlight=None):
        deadline = time.monotonic() + timeout
        actual = None
        while time.monotonic() < deadline:
            if self.capture.exists():
                parts = self.capture.read_bytes().decode().split('\0')
                if len(parts) == 4:
                    actual = parts
                    if (parts[:2] == [buffer, suggestion]
                            and (cursor is None or parts[3] == str(cursor))
                            and (highlight is None or highlight in parts[2])):
                        return parts[2]
                    if parts[0] == buffer:
                        # Only inspect async state after typed input has reached ZLE.
                        # Queuing inspection keys earlier suppresses suggestion fetching.
                        self.send(b'\x18\x02')
            self.drain()
        raise AssertionError(f'Expected {(buffer, suggestion)!r}, got {actual!r}\n{self.output[-3000:]!r}')

    def close(self):
        try:
            os.kill(self.pid, signal.SIGKILL)
            os.waitpid(self.pid, 0)
        finally:
            os.close(self.fd)


with tempfile.TemporaryDirectory(prefix='moodsh-suggestions-') as tmp:
    root = Path(tmp)
    history = ['echo moodsh-order old', 'echo moodsh-order latest',
               'echo moodsh-unicode café', 'echo moodsh-literal[one] $(touch OWNED)',
               'printf done > executed', 'x' * 257 + ' suffix', 'echo fixture-end']
    session = Session(root, history)
    try:
        session.send(b'echo moodsh-order ')
        session.state('echo moodsh-order ', 'latest', highlight='fg=8')
        session.send(b'\x1b[C')  # Right accepts at end of input, without executing.
        session.state('echo moodsh-order latest', '')
        session.send(b'\x15')
        session.state('', '')
        session.send('echo moodsh-unicode c'.encode())
        session.state('echo moodsh-unicode c', 'afé')
        session.send(b'\x7f')
        session.state('echo moodsh-unicode ', 'café')
        session.send(b'\x15echo moodsh-literal[one] ')
        session.state('echo moodsh-literal[one] ', '$(touch OWNED)')
        assert not (root / 'OWNED').exists(), 'Rendering history executed a command'
        session.send(b'\x03')  # Cancel clears both typed text and ghost text.
        session.state('', '')
        session.send(b'printf do')
        session.state('printf do', 'ne > executed')
        # Moving right in the middle must not accept the suggestion.
        session.send(b'\x01\x1b[C')
        session.state('printf do', 'ne > executed', cursor=1)
        assert not (root / 'executed').exists()
        session.send(b'\x05\x1b[C')
        session.state('printf done > executed', '')
        assert not (root / 'executed').exists(), 'Acceptance must not execute'
        session.send(b'\r')
        session.state('', '')
        assert (root / 'executed').read_text() == 'done'
        session.send(b'moodsh theme set em\t')
        session.state('moodsh theme set ember ', '')
        session.send(b'\x15cat ~/.zsh_h\t')
        session.state('cat ~/.zsh_history ', '')
        session.send(b'\x15echo no-history-match')
        session.state('echo no-history-match', '')
        session.send(b'\x15' + b'x' * 257)
        session.state('x' * 257, '')
        print('PASS Zsh PTY: ghost text, latest history, Right acceptance, cursor motion, Unicode, deletion, cancellation, literal metacharacters, Tab completion, long input')
    finally:
        session.close()

    for label, kwargs in [
        ('opt-out', {'flags': ('--no-suggestions',)}),
        ('NO_COLOR', {'no_color': True}),
        ('dumb terminal', {'term': 'dumb'}),
        ('existing engine', {'before': '_zsh_autosuggest_start() { :; }\n'}),
    ]:
        session = Session(root, history, **kwargs)
        try:
            session.send(b'echo moodsh-order ')
            session.state('echo moodsh-order ', '')
            print('PASS Zsh PTY:', label)
        finally:
            session.close()

    session = Session(root, history, before="""
_custom_right() { BUFFER+='!'; CURSOR=$#BUFFER; }
zle -N custom-right _custom_right
bindkey '^[[C' custom-right
ZSH_AUTOSUGGEST_HIGHLIGHT_STYLE='fg=cyan'
""")
    try:
        session.send(b'echo moodsh-order ')
        session.state('echo moodsh-order ', 'latest', highlight='fg=cyan')
        session.send(b'\x1b[C')
        session.state('echo moodsh-order !', '')
        print('PASS Zsh PTY: custom Right binding and highlight preference preserved')
    finally:
        session.close()
