# Mood Shell

**Make your shell feel like yours.**

A small shell customization toolkit written in Rust. Pick a mood, tune its colors, and get the same prompt style in Zsh, Bash, or PowerShell. Built for macOS, Windows, and Linux.

```text
~/code/moodsh > git status

# Or give your prompt a little more room:
dusk ~/code/moodsh
>
```

Version **0.1.0** starts with the essentials: five palettes, a live picker, two layouts, and a readable prompt that shows when a command fails. No required fonts, account, daemon, or telemetry.

## Install

Download an archive from [Releases](https://github.com/mager/moodsh/releases), extract it, and place `moodsh` (Windows: `moodsh.exe`) in a directory on your `PATH`.

Release binaries: Apple Silicon macOS, Intel macOS, x64 Linux (glibc), and x64 Windows. Other Rust-supported architectures can build from source. Windows support is native and does not require WSL. macOS binaries are unsigned; building from source is an alternative if macOS blocks a downloaded binary.

With a current stable Rust toolchain and Git:

```sh
cargo install --git https://github.com/mager/moodsh --tag v0.1.0 --locked
```

For development:

```sh
git clone https://github.com/mager/moodsh.git
cd moodsh
cargo install --path . --locked
```

Confirm `moodsh --version` works before adding your shell integration.

## Connect your shell

Use one prompt engine at a time: remove or comment out an existing Starship/Oh My Posh init line. With Oh My Zsh, use `ZSH_THEME=""` and put Mood Shell's init **after** Oh My Zsh loads. Your other plugins can stay.

### Zsh · macOS and Linux

Add this to `~/.zshrc`, then open a new shell:

```sh
eval "$(moodsh init zsh)"
```

### Bash · macOS and Linux

Add this to `~/.bashrc` (or your existing interactive Bash startup file), then open a new shell:

```sh
eval "$(moodsh init bash)"
```

macOS login Bash reads `~/.bash_profile`; make sure that file sources `~/.bashrc` if you use both. Bash 3.2 and newer are supported.

### PowerShell 7 · Windows, macOS, and Linux

Add this to your PowerShell profile (`$PROFILE`), then open a new shell:

```powershell
Invoke-Expression (& moodsh init powershell | Out-String)
```

If your profile does not exist, create its parent directory and an empty file before editing it. Mood Shell replaces PowerShell's `prompt` function and preserves `$LASTEXITCODE` after rendering.

`moodsh init` only prints the integration script; it does not edit your startup files. To disconnect, remove its init line and restart the shell.

## Find your mood

```sh
moodsh customize
```

Use **↑/↓** or **j/k** to browse live previews, **Tab** to switch layouts, **Enter** to save, and **Esc** to cancel. The picker also shows a failed-command preview. It requires an interactive terminal; the commands below work in scripts.

| Mood | Accent | Feel |
| --- | --- | --- |
| `dusk` | `#C4A7E7` | Soft lavender after dark |
| `aurora` | `#A6E3A1` | Green and icy cyan |
| `ember` | `#FFB454` | Warm amber |
| `ocean` | `#7DCFFF` | Clear blue |
| `paper` | `#5C3599` | Purple and ink for light terminals |

```sh
moodsh theme list
moodsh theme preview ocean
moodsh theme preview dusk --layout two-line
moodsh theme set ocean
moodsh theme set dusk --accent '#FF77CC' --path '#E6E6FA' --layout two-line
```

Color overrides: `--accent`, `--path`, `--muted`, and `--error`. Each accepts a quoted `#RRGGBB`. Setting a built-in mood resets its palette and layout before applying your overrides. `muted` colors the mood name in the two-line layout.

Saved changes appear at the next prompt, including in other shells using the same config. A theme preview never saves changes. For your exact current palette, run `moodsh theme preview` without a name.

## Your own palette

```sh
moodsh config path
moodsh config show
```

The first command prints the config location. The second prints the effective configuration. Before you save a mood, Mood Shell uses `dusk` without creating a file.

```toml
[mood]
name = "afterhours"
layout = "two-line"

[mood.palette]
accent = "#C4A7E7"
path = "#E0DEF4"
muted = "#908CAA"
error = "#EB6F92"
```

Copy [the example palette](themes/custom.toml) to your config path or edit your saved file. All shown fields are required. `layout` is `compact` or `two-line`; names accept letters, numbers, hyphens, and underscores. Unknown keys and invalid colors produce an error rather than being silently ignored.

Default locations:

- macOS: `~/Library/Application Support/moodsh/config.toml`
- Linux: `$XDG_CONFIG_HOME/moodsh/config.toml`, or `~/.config/moodsh/config.toml`
- Windows: `%APPDATA%\moodsh\config.toml`

Set `MOODSH_CONFIG` to an absolute file path to use a different config. Config writes replace the file atomically. A malformed existing config is never silently overwritten; fix it or move it aside before saving a new mood.

## Diagnostics and scope

```sh
moodsh doctor
moodsh prompt --no-color
```

A 24-bit color terminal gives the intended palette. Set `NO_COLOR` to disable colors; `TERM=dumb` also uses plain output. Prompt symbols are ASCII, so no Nerd Font is required.

In 0.1, a mood styles **the prompt**. Terminal backgrounds, command syntax highlighting, autosuggestions, plugin management, Fish support, and animations are future work. Mood Shell augments your existing shell; it is not a command interpreter or an MCP server.

Zsh integration disables `PROMPT_SUBST` to keep directory names from being evaluated as shell code. Bash escapes prompt metacharacters. Both preserve existing prompt hooks, though another prompt engine can overwrite their output. Keep Mood Shell as the only prompt renderer. If the config becomes invalid, the shell hook falls back to a simple usable prompt and prints the error.

## Development and releases

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked
python3 scripts/test_shells.py target/debug/moodsh
```

On Windows, use `python` and `target/debug/moodsh.exe`. The shell integration tests exercise available supported shells; CI requires PowerShell on all three operating systems, plus Bash/Zsh where available.

Versions follow [Semantic Versioning](https://semver.org/), starting at `0.1.0`. Before 1.0, breaking changes increment the minor version and compatible fixes increment the patch version. Update `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, and `.github/release-notes.md` before tagging `vX.Y.Z`. Pushing a tag builds release archives and publishes them with SHA-256 checksums. The workflow rejects a tag that does not match the package version.

[MIT](LICENSE).
