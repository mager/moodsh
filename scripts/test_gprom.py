"""Check branch resolution and rebase only disposable local Git repositories."""
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import sys
import tempfile

binary = str(Path(sys.argv[1]).resolve())
with tempfile.TemporaryDirectory(prefix='moodsh-gprom-') as directory:
    root = Path(directory)
    env = dict(os.environ, GIT_CONFIG_NOSYSTEM='1', GIT_CONFIG_GLOBAL=os.devnull,
               GIT_AUTHOR_NAME='Test', GIT_AUTHOR_EMAIL='test@example.com',
               GIT_COMMITTER_NAME='Test', GIT_COMMITTER_EMAIL='test@example.com',
               GIT_TERMINAL_PROMPT='0', MOODSH_CONFIG=str(root / 'config.toml'))

    def call(args, cwd=root, ok=True):
        result = subprocess.run(args, cwd=cwd, env=env, text=True, capture_output=True, timeout=30)
        if ok:
            assert result.returncode == 0, (args, result.stdout, result.stderr)
        return result

    def git(*args, cwd=root):
        return call(['git', *args], cwd).stdout.strip()

    repo = root / 'resolution'
    git('init', '--initial-branch=feature', str(repo))
    git('commit', '--allow-empty', '-m', 'seed', cwd=repo)
    helper = [binary, '__git-main-branch']
    assert 'inside a Git repository' in call(helper, ok=False).stderr
    assert 'requires an origin remote' in call(helper, cwd=repo, ok=False).stderr
    git('remote', 'add', 'origin', str(root / 'nonexistent-remote'), cwd=repo)
    assert 'Cannot resolve' in call(helper, cwd=repo, ok=False).stderr
    git('branch', 'master', cwd=repo)
    assert call(helper, cwd=repo).stdout.strip() == 'master'
    git('branch', 'main', cwd=repo)
    assert call(helper, cwd=repo).stdout.strip() == 'main'
    git('update-ref', 'refs/remotes/origin/master', 'HEAD', cwd=repo)
    assert call(helper, cwd=repo).stdout.strip() == 'master'
    branch = 'release/$(echo_owned)'
    git('update-ref', f'refs/remotes/origin/{branch}', 'HEAD', cwd=repo)
    git('symbolic-ref', 'refs/remotes/origin/HEAD', f'refs/remotes/origin/{branch}', cwd=repo)
    assert call(helper, cwd=repo).stdout.strip() == branch
    git('symbolic-ref', 'refs/remotes/origin/HEAD', 'refs/remotes/origin/missing', cwd=repo)
    assert call(helper, cwd=repo).stdout.strip() == 'master'
    git('symbolic-ref', 'refs/remotes/origin/HEAD', f'refs/remotes/origin/{branch}', cwd=repo)
    print('PASS branch resolution: outside repo, missing origin, unknown, main/master, remote priority, custom HEAD, stale HEAD')

    shells = []
    if os.name != 'nt':
        shells.extend(s for s in ('bash', 'zsh') if shutil.which(s))
    if shutil.which('pwsh'):
        shells.append('powershell')
    elif os.environ.get('CI'):
        raise SystemExit('CI must test PowerShell gprom')

    for shell in shells:
        init = root / ('init.ps1' if shell == 'powershell' else 'init.sh')
        init.write_text(call([binary, 'init', shell, '--shortcuts', 'git']).stdout)
        assert 'function gprom' not in call([binary, 'init', shell]).stdout
        assert 'function global:gprom' not in call([binary, 'init', shell]).stdout
        if shell == 'powershell':
            quote = lambda s: "'" + str(s).replace("'", "''") + "'"
            source = f'. {quote(init)}\n'
            command = [shutil.which('pwsh'), '-NoProfile', '-NonInteractive', '-Command']
            def run(script, cwd=repo, ok=True):
                return call(command + [script], cwd, ok)
            mocked = source + source + "function global:git { ConvertTo-Json -InputObject @($args) -Compress; $global:LASTEXITCODE=0 }\ngprom --autostash\n"
            args = json.loads(run(mocked).stdout.strip())
            assert args == ['pull', '--rebase', 'origin', branch, '--autostash'], args
            collision = "Set-Alias gprom Write-Output\n" + source + "gprom USER_ALIAS"
            assert run(collision).stdout.strip() == 'USER_ALIAS'
            collision = "function global:gprom { 'USER_FUNCTION' }\n" + source + 'gprom'
            assert run(collision).stdout.strip() == 'USER_FUNCTION'
            failed = source + "function global:git { 'PULL_RAN'; $global:LASTEXITCODE=0 }\ngprom"
            result = run(failed, cwd=root, ok=False)
            assert result.returncode != 0 and 'PULL_RAN' not in result.stdout, result
            failed = source + "function global:git { $global:LASTEXITCODE=7 }\n$failed=$false; try { gprom } catch { $failed=$true }; if (-not $failed -or $global:LASTEXITCODE -ne 7) { throw 'Git failure was hidden' }; 'EXPECTED_FAILURE'"
            assert run(failed).stdout.strip() == 'EXPECTED_FAILURE'
        else:
            source = ('shopt -s expand_aliases\n' if shell == 'bash' else '') + f'source {shlex.quote(str(init))}\n'
            command = [shutil.which(shell), '--noprofile', '--norc'] if shell == 'bash' else [shutil.which(shell), '-f']
            def run(script, cwd=repo, ok=True):
                result = subprocess.run(command, input=script, cwd=cwd, env=env, text=True, capture_output=True, timeout=30)
                if ok:
                    assert result.returncode == 0, result.stdout + result.stderr
                return result
            mocked = source + source + 'git() { printf "%s\\n" "$@"; return 7; }\ngprom --autostash\nprintf "STATUS=%s\\n" "$?"\n'
            assert run(mocked).stdout.splitlines() == ['pull', '--rebase', 'origin', branch, '--autostash', 'STATUS=7']
            collision = "alias gprom='printf USER_ALIAS'\n" + source + 'gprom\n'
            assert run(collision).stdout == 'USER_ALIAS'
            collision = "function gprom { printf USER_FUNCTION; }\n" + source + 'gprom\n'
            assert run(collision).stdout == 'USER_FUNCTION'
            failed = source + "git() { printf PULL_RAN; }\ngprom\n"
            result = run(failed, cwd=root, ok=False)
            assert result.returncode != 0 and 'PULL_RAN' not in result.stdout, result

        # Run the actual shortcut, without mocks or network, against main and master.
        for primary in ('main', 'master'):
            seed = root / f'{shell}-{primary}-seed'
            remote = root / f'{shell}-{primary}-remote.git'
            work = root / f'{shell}-{primary}-work'
            git('init', f'--initial-branch={primary}', str(seed))
            git('commit', '--allow-empty', '-m', 'seed', cwd=seed)
            git('clone', '--bare', str(seed), str(remote))
            git('clone', str(remote), str(work))
            git('switch', '-c', 'feature', cwd=work)
            (work / 'local.txt').write_text('local change\n')
            git('add', '.', cwd=work)
            git('commit', '-m', 'local change', cwd=work)
            (seed / 'remote.txt').write_text('remote change\n')
            git('add', '.', cwd=seed)
            git('commit', '-m', 'remote change', cwd=seed)
            git('push', str(remote), primary, cwd=seed)
            run(source + 'gprom\n', cwd=work)
            assert git('branch', '--show-current', cwd=work) == 'feature'
            assert git('rev-parse', 'HEAD^', cwd=work) == git('rev-parse', 'HEAD', cwd=seed)
            assert (work / 'local.txt').read_text() == 'local change\n'
            assert (work / 'remote.txt').read_text() == 'remote change\n'
        print(f'PASS {shell}: collisions, opt-in, repeated init, literal branch/arguments, errors, real main/master rebase')
