# Mood Shell

**Make your shell feel like yours.**

A small shell customization toolkit written in Rust. Pick a mood, tune its colors, and get the same prompt style in Zsh, Bash, or PowerShell. Built for macOS, Windows, and Linux.

```text
~/code/moodsh > git status

# Or give your prompt a little more room:
dusk ~/code/moodsh
>
```

Version **0.4.0** adds Tab completion when you connect your shell. Start with five palettes, tune hex colors in the live picker, and export your own Markdown theme. Two layouts and failed-command status keep the prompt readable. No required fonts, account, daemon, or telemetry.

[Website](https://moodsh.vercel.app) · [Theme gallery](https://moodsh.vercel.app/themes/) · [Make and share a theme](https://moodsh.vercel.app/themes/share/)

## Install

**Rust and Cargo are not required to use Mood Shell.** Download an archive from [Releases](https://github.com/mager/moodsh/releases) and extract it. Choose the archive for your platform (not GitHub's “Source code” download):

| Platform | Archive name ends with |
| --- | --- |
| macOS, Apple Silicon | `aarch64-apple-darwin.tar.gz` |
| macOS, Intel | `x86_64-apple-darwin.tar.gz` |
| Linux, x64 (glibc) | `x86_64-unknown-linux-gnu.tar.gz` |
| Windows, x64 | `x86_64-pc-windows-msvc.zip` |

On **macOS or Linux**, open a terminal in the extracted folder and run:

```sh
mkdir -p "$HOME/.local/bin"
install -m 755 ./moodsh "$HOME/.local/bin/moodsh"
export PATH="$HOME/.local/bin:$PATH"
moodsh --version
```

The `export` applies to this session. Add it to your shell startup file before the Mood Shell init line to keep it in future sessions.

On **Windows**, open PowerShell in the extracted folder and run:

```powershell
$dest = Join-Path $env:LOCALAPPDATA 'Programs\moodsh'
New-Item -ItemType Directory -Force $dest | Out-Null
Copy-Item .\moodsh.exe (Join-Path $dest 'moodsh.exe')
$env:Path = "$dest;$env:Path"
moodsh --version
```

For future sessions, add that directory to your user `Path` using Windows' **Edit environment variables for your account**, then reopen your terminal.

Release binaries: Apple Silicon macOS, Intel macOS, x64 Linux (glibc), and x64 Windows. Other Rust-supported architectures can build from source. Windows support is native and does not require WSL. macOS binaries are unsigned; building from source is an alternative if macOS blocks a downloaded binary.

### Build from source (optional)

With a current stable [Rust toolchain](https://rust-lang.org/tools/install/) and Git:

```sh
cargo install --git https://github.com/mager/moodsh --tag v0.4.0 --locked
```

**`zsh: command not found: cargo`?** Cargo is Rust's build tool, not a built-in shell command. Use the prebuilt download above, or install Rust first. If Rust is already installed in `~/.cargo`, load its PATH in your current Zsh/Bash session:

```sh
source "$HOME/.cargo/env"
cargo --version
```

If `~/.cargo/env` does not exist, follow the Rust installation link or use a prebuilt binary. If Mood Shell is already installed with Cargo, `~/.cargo/bin/moodsh customize` works even before fixing PATH. Neither installing Mood Shell nor running the picker connects it to your shell automatically.

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

### Complete commands with Tab

The same init line now registers completion for `moodsh` commands, flags, layouts, and built-in mood names. Zsh and Bash also complete theme file paths. Try `moodsh theme set em` and press **Tab** to complete `ember`. The completion script is generated from the CLI's command definitions, so it stays in sync with the installed version.

For Zsh, Mood Shell also initializes Zsh's standard completion system when it is not already active. This restores normal command and path completion after removing Oh My Zsh. Bash and PowerShell retain their existing command and path completion; Mood Shell adds its own command definitions to those shells. Completion runs when you press Tab, while prompt rendering still uses the same small hook.

This release provides **Tab completion**. Inline suggestions from your command history while you type are a separate feature and are not included yet.

## Find your mood

```sh
moodsh customize
```

Use **↑/↓** or **j/k** to browse live previews and **Tab** to switch layouts. Press **1–4** to edit a color:

| Key | Color | What it changes |
| --- | --- | --- |
| `1` | Accent | Prompt arrow |
| `2` | Path | Directory text |
| `3` | Muted | Mood name in the two-line layout |
| `4` | Error | Failed-command status and arrow |

Type or paste a `#RRGGBB` color (the `#` is optional). Typing replaces the current value; **Backspace** edits it and **Ctrl-U** clears it. A complete color updates the success and failed-command previews immediately. **Enter** applies the color to your draft and returns to browsing; **Esc** discards just that color edit.

From browsing, **Enter** saves the whole draft and **Esc** cancels all changes. **Ctrl-C** cancels from either view. You can switch moods and return without losing draft color edits. Your exact saved palette is included as a `(saved)` choice when it differs from the built-ins. Nothing is written until you save from browsing.

The picker requires an interactive terminal of at least 80 columns by 24 rows; it pauses for resizing in smaller windows. `NO_COLOR` or `TERM=dumb` disables the color preview, but you can still edit values. The commands below work in scripts.

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

## Create and share a theme in Markdown

Start from a built-in mood:

```sh
moodsh theme new afterhours.md --from ocean
```

This creates an editable document with colors, layout, and an explanation of each field. The theme name defaults to the filename without its extension; use `--name afterhours` if your filename contains spaces. Names use 1–48 ASCII letters, numbers, hyphens, or underscores.

Or design your palette with `moodsh customize`, save it, then export it:

```sh
moodsh theme export afterhours.md --name afterhours
```

Both commands leave your active prompt unchanged and refuse to overwrite existing files. Use a new filename when exporting a revision.

A theme is an ordinary Markdown document with exactly one `moodsh` code block. Write about your inspiration, credit the author, or explain which terminal background suits the palette. The block contains the same TOML fields as the runtime config:

````markdown
# Afterhours

Pink and lavender for a dark terminal, with a little room to breathe.

```moodsh
[mood]
name = "afterhours"
layout = "two-line"

[mood.palette]
accent = "#FF77CC"
path = "#E0DEF4"
muted = "#908CAA"
error = "#EB6F92"
```
````

Save a shared document locally, then inspect its preview before applying it:

```sh
moodsh theme preview --file afterhours.md
moodsh theme apply afterhours.md
```

Previewing writes nothing. Applying validates and copies the palette into your runtime config; it never executes prose or shell code. Subsequent edits to the Markdown file take effect only when you apply it again. You can also fine-tune an applied theme in `moodsh customize` and export a new document.

Try the included [Afterhours theme](themes/afterhours.md). See the [theme format guide](themes/README.md) for supported fences and validation rules. Files are local UTF-8 Markdown, at most 64 KiB. Links and URLs inside a theme are never fetched. The Markdown file travels with the theme's explanation; your prompt reads only its saved TOML configuration.

## Runtime configuration

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

A mood styles **the prompt**. Tab completion assists with typing commands; it does not change terminal colors. Terminal backgrounds, command syntax highlighting, inline history suggestions, plugin management, Fish support, and animations are future work. Mood Shell augments your existing shell; it is not a command interpreter or an MCP server.

A blinking or pulsing cursor comes from your terminal emulator's settings, not Mood Shell. This is an early public release for people who want a small, customizable prompt; it does not replace the plugin features of a full shell framework.

Zsh integration disables `PROMPT_SUBST` to keep directory names from being evaluated as shell code. Bash escapes prompt metacharacters. Both preserve existing prompt hooks, though another prompt engine can overwrite their output. Keep Mood Shell as the only prompt renderer. If the config becomes invalid, the shell hook falls back to a simple usable prompt and prints the error.

## Development and releases

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked
python3 scripts/test_shells.py target/debug/moodsh
python3 scripts/test_picker.py target/debug/moodsh
```

On Windows, use `python` and `target/debug/moodsh.exe` for the shell integration tests. The PTY picker script runs on macOS/Linux; picker state and input tests run natively on all three platforms. The shell integration tests exercise available supported shells; CI requires PowerShell on all three operating systems, plus Bash/Zsh where available.

Versions follow [Semantic Versioning](https://semver.org/), starting at `0.1.0`. New features increment the minor version; compatible fixes increment the patch version. Before 1.0, breaking changes also increment the minor version and are called out in the changelog. Update `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, and `.github/release-notes.md` before tagging `vX.Y.Z`. Pushing a tag builds release archives and publishes them with SHA-256 checksums. The workflow rejects a tag that does not match the package version.

[MIT](LICENSE).

## Website and community

Browse and remix palettes in the [theme gallery](https://moodsh.vercel.app/themes/). The browser theme maker imports and downloads portable Markdown, with live previews of all four colors and both layouts. Community submissions open on GitHub for review and author credit. See [web/README.md](web/README.md) to develop the site or review a submission.
