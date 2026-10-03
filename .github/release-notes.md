Mood Shell 0.6.0 adds **gprom** to the opt-in Git shortcut set on Zsh, Bash, and PowerShell 7.

`gprom` runs `git pull --rebase origin <main-branch>`. It resolves the branch when invoked, preferring a valid local `origin/HEAD`, then common names including `main` and `master`. Missing repository, origin, or branch metadata stops the shortcut before a pull. Extra arguments such as `--autostash` are forwarded literally. A pull fetches and rebases the current branch using normal Git behavior.

Keep `--shortcuts git` in your existing init line and open a new shell after upgrading. Existing `gprom` aliases, functions, and executables are preserved. `moodsh shortcuts git` shows all 26 shortcuts.

PowerShell shortcuts raise a terminating error on Git failure and preserve `$LASTEXITCODE`; scripts that handle Git exit codes themselves should invoke `git` directly. Git is installed separately. No new Rust dependencies, configuration changes, or per-prompt work.

The release includes native Apple Silicon and Intel macOS, x64 Linux, and x64 Windows binaries with SHA-256 checksums. Rust is not required. Prompt themes, colors, layouts, and Tab completion remain compatible.
