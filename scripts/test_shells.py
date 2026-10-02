"""Exercise real prompt expansion, hook preservation, and command status."""
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

binary = str(Path(sys.argv[1]).resolve())

with tempfile.TemporaryDirectory(prefix="moodsh-test-") as root:
    root = Path(root)
    # Command-substitution and prompt-escape payloads must remain literal text.
    cwd = root / '$(touch OWNED_DOLLAR)`touch OWNED_BACKTICK`%F{red}'
    cwd.mkdir()
    env = dict(os.environ, MOODSH_CONFIG=str(root / "config.toml"), NO_COLOR="1", TERM="dumb")
    env["PATH"] = str(Path(binary).parent) + os.pathsep + env.get("PATH", "")
    subprocess.run([binary, "theme", "set", "dusk"], env=env, check=True, capture_output=True)
    tested = []
    for shell in ("bash", "zsh") if os.name != "nt" else ():
        exe = shutil.which(shell)
        if not exe:
            continue
        integration = subprocess.check_output([binary, "init", shell], env=env, text=True)
        init_file = root / (shell + ".sh")
        init_file.write_text(integration)
        # Existing hooks survive, and repeated init does not duplicate our hook.
        hook = "PROMPT_COMMAND='printf HOOK >&2'" if shell == "bash" else "old_hook() { printf HOOK >&2; }; precmd_functions=(old_hook)"
        script = f"{hook}\nsource '{init_file}'\nsource '{init_file}'\nfalse\nprintf 'DONE\\n'\nexit\n"
        args = [exe, "--noprofile", "--norc", "-i"] if shell == "bash" else [exe, "-f", "-i"]
        proc = subprocess.run(args, input=script, text=True, env=env, cwd=cwd, capture_output=True, timeout=15)
        output = proc.stdout + proc.stderr
        assert proc.returncode == 0, output
        assert "[1]" in output, output
        assert "HOOK" in output, output
        assert not (cwd / "OWNED_DOLLAR").exists(), output
        assert not (cwd / "OWNED_BACKTICK").exists(), output
        # Verify idempotency directly without relying on terminal echo.
        check = "printf '%s' \"$PROMPT_COMMAND\"" if shell == "bash" else "print -r -- $precmd_functions"
        plain_args = [exe, "--noprofile", "--norc", "-c"] if shell == "bash" else [exe, "-f", "-c"]
        hooks = subprocess.check_output(plain_args + [f"source '{init_file}'; source '{init_file}'; {check}"], env=env, text=True)
        assert hooks.count("_moodsh_prompt" if shell == "bash" else "_moodsh_precmd") == 1, hooks
        if shell == "zsh":
            status = subprocess.check_output(plain_args + [f"source '{init_file}'; false; _moodsh_precmd; print -r -- $?"], env=env, text=True)
            assert status.strip() == "0", "Zsh must continue running subsequent precmd hooks"
            completion = subprocess.check_output(plain_args + [f"source '{init_file}'; print -r -- ${{_comps[moodsh]}}"], env=env, text=True)
            assert completion.strip() == "_moodsh", completion
        else:
            completion = subprocess.check_output(plain_args + [f"source '{init_file}'; COMP_WORDS=(moodsh theme set em); COMP_CWORD=3; _moodsh moodsh em set; printf '%s\\n' \"${{COMPREPLY[@]}}\""], env=env, text=True)
            assert completion.splitlines() == ["ember"], completion
            version = subprocess.check_output([exe, "--version"], text=True)
            if "version 3." not in version:
                array_check = f"PROMPT_COMMAND=('echo old'); source '{init_file}'; source '{init_file}'; printf '%s\\n' \"${{PROMPT_COMMAND[@]}}\""
                hooks = subprocess.check_output(plain_args + [array_check], env=env, text=True)
                assert hooks.splitlines() == ["_moodsh_prompt", "echo old"], hooks
        print(f"PASS {shell}: prompt, status, hooks, repeated init, Tab completion")
        tested.append(shell)
    pwsh = shutil.which("pwsh")
    if pwsh:
        integration = subprocess.check_output([binary, "init", "powershell"], env=env, text=True)
        init_file = root / "init.ps1"
        init_file.write_text(integration)
        quoted = str(init_file).replace("'", "''")
        script = f". '{quoted}'\n$matches = [System.Management.Automation.CommandCompletion]::CompleteInput('moodsh th', 9, $null).CompletionMatches\nif (-not ($matches.CompletionText -contains 'theme')) {{ throw 'Mood Shell Tab completion is missing' }}\n$global:LASTEXITCODE = 9\nWrite-Error 'expected test error' -ErrorAction Continue\n$result = prompt\nif ($result -notmatch '\\[9\\]') {{ throw 'Lost failure status' }}\nif ($global:LASTEXITCODE -ne 9) {{ throw 'Changed LASTEXITCODE' }}\nWrite-Output $result\nexit 0"
        proc = subprocess.run([pwsh, "-NoLogo", "-NoProfile", "-NonInteractive", "-Command", script], cwd=cwd, env=env, text=True, capture_output=True, timeout=20)
        assert proc.returncode == 0, proc.stdout + proc.stderr
        assert "OWNED_DOLLAR" in proc.stdout
        assert not (cwd / "OWNED_DOLLAR").exists()
        assert not (cwd / "OWNED_BACKTICK").exists()
        print("PASS powershell: prompt rendering, failure status, LASTEXITCODE, Tab completion")
        tested.append("powershell")
    if not tested:
        raise SystemExit("No supported shell was available for integration tests")
    if os.environ.get("CI") and "powershell" not in tested:
        raise SystemExit("CI must exercise PowerShell on every platform")
