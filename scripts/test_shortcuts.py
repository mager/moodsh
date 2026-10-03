"""Exercise opt-in shortcuts, collisions, literal arguments, and exit status."""
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import tempfile

binary = str(Path(sys.argv[1]).resolve())
catalog = subprocess.check_output([binary, 'shortcuts', 'git'], text=True)
assert 'gap    git apply' in catalog
payload = 'message with spaces; $(touch OWNED) `touch OWNED` "quotes"'
with tempfile.TemporaryDirectory(prefix='moodsh-shortcuts-') as directory:
    root = Path(directory)
    env = dict(os.environ, MOODSH_CONFIG=str(root / 'config.toml'))
    for shell in ('bash', 'zsh') if os.name != 'nt' else ():
        exe = shutil.which(shell)
        if not exe:
            continue
        init = root / 'init.sh'
        source = subprocess.check_output([binary, 'init', shell, '--shortcuts', 'git'], text=True)
        default = subprocess.check_output([binary, 'init', shell], text=True)
        assert "alias gst='git status'" not in default
        init.write_text(source)
        args = [exe, '--noprofile', '--norc'] if shell == 'bash' else [exe, '-f']
        setup = 'shopt -s expand_aliases\n' if shell == 'bash' else ''
        script = setup + f"""
alias ga='printf USER_ALIAS'
gb() {{ printf USER_FUNCTION; }}
source {shlex.quote(str(init))}
source {shlex.quote(str(init))}
ga
printf '\\n'
gb
printf '\\n'
g --version
git() {{ printf '<%s>\\n' "$@"; return 7; }}
gcmsg {shlex.quote(payload)}
printf 'STATUS=%s\\n' "$?"
gst
printf 'STATUS=%s\\n' "$?"
"""
        result = subprocess.run(args, input=script, env=env, cwd=root, text=True, capture_output=True, timeout=20)
        assert result.returncode == 0, result.stderr
        assert 'USER_ALIAS\nUSER_FUNCTION\ngit version' in result.stdout, result.stdout
        assert f'<commit>\n<--message>\n<{payload}>\nSTATUS=7' in result.stdout, result.stdout
        assert '<status>\nSTATUS=7' in result.stdout, result.stdout
        assert not (root / 'OWNED').exists()
        print(f'PASS {shell}: opt-in, aliases/functions preserved, repeated init, literal arguments, status')
    pwsh = shutil.which('pwsh')
    if pwsh:
        source = subprocess.check_output([binary, 'init', 'powershell', '--shortcuts', 'git'], text=True)
        assert 'function global:gst' not in subprocess.check_output([binary, 'init', 'powershell'], text=True)
        init = root / 'init.ps1'
        init.write_text(source)
        quote = lambda value: "'" + str(value).replace("'", "''") + "'"
        script = f"""
$ErrorActionPreference = 'Stop'
function global:ga {{ 'USER_FUNCTION' }}
Set-Alias gb Write-Output
$original = (Get-Alias gc).Definition
. {quote(init)}
. {quote(init)}
if ((ga) -ne 'USER_FUNCTION') {{ throw 'Overwrote function' }}
if ((Get-Alias gb).Definition -ne 'Write-Output') {{ throw 'Overwrote alias' }}
if ((Get-Alias gc).Definition -ne $original) {{ throw 'Overwrote PowerShell built-in' }}
if ((g --version) -notmatch '^git version') {{ throw 'Native Git invocation failed' }}
$failed = $false
try {{ g moodsh-nonexistent-test-command 2>$null }} catch {{ $failed = $true }}
if (-not $failed -or $global:LASTEXITCODE -eq 0) {{ throw 'Git failure was hidden' }}
function global:git {{ ConvertTo-Json -InputObject @($args) -Compress; $global:LASTEXITCODE = 0 }}
$result = gcmsg {quote(payload)}
if ($global:LASTEXITCODE -ne 0) {{ throw 'Lost exit code' }}
$result
$result = gst -v
if ($global:LASTEXITCODE -ne 0) {{ throw 'Lost status exit code' }}
$result
"""
        result = subprocess.run([pwsh, '-NoLogo', '-NoProfile', '-NonInteractive', '-Command', script], env=env, cwd=root, text=True, capture_output=True, timeout=20)
        assert result.returncode == 0, result.stdout + result.stderr
        lines = result.stdout.strip().splitlines()
        assert json.loads(lines[-2]) == ['commit', '--message', payload], result.stdout
        assert json.loads(lines[-1]) == ['status', '-v'], result.stdout
        assert not (root / 'OWNED').exists()
        print('PASS powershell: collisions, repeated init, native Git, literal arguments, status')
    elif os.environ.get('CI'):
        raise SystemExit('CI must test PowerShell shortcuts')
