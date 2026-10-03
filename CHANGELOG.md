# Changelog

Versions follow Semantic Versioning. New features increment the minor version; compatible fixes increment the patch version. Before 1.0, breaking changes also increment the minor version and are called out here. Git tags use `vMAJOR.MINOR.PATCH` and must match `Cargo.toml`.

## 0.6.0 — 2026-10-03

- Add opt-in `gprom` on Zsh, Bash, and PowerShell: pull and rebase from origin’s main branch.
- Resolve the branch at invocation from local Git metadata, preferring a valid origin/HEAD and falling back to common branch names. Stop before pulling if discovery fails.
- Preserve existing commands, pass branch names and extra arguments literally, and retain native Git failure handling.
- Exercise branch resolution and real main/master rebases against disposable local repositories in cross-platform CI. No new dependencies or prompt-hook changes.

## 0.5.0 — 2026-10-02

- Add 25 opt-in Git shortcuts with familiar Oh My Zsh names, enabled with `moodsh init <shell> --shortcuts git`.
- Show every expansion with `moodsh shortcuts git`; preserve existing aliases, functions, built-ins, and executables on Zsh, Bash, and PowerShell.
- Surface failed PowerShell shortcuts as terminating errors while preserving Git’s exit code.
- Forward arguments without re-evaluating shell text. Keep shortcuts independent of theme files, configuration, and prompt rendering.
- Test collision handling, repeated startup, quoted arguments, native Git invocation, and exit codes in real shells across CI platforms.

## Website 0.1.0 — 2026-10-02

The website is versioned independently; the CLI remains at 0.4.0.

- Launch moodsh.vercel.app with interactive palette previews and platform-specific installation instructions.
- Add a searchable theme gallery with portable Markdown downloads and author credit.
- Make, import, remix, and download themes in the browser with live color and layout previews.
- Open community submissions as prepared GitHub issues for review; no website account or database is required.
- Keep preview backgrounds separate from the prompt-only theme format.

## 0.4.0 — 2026-10-02

- Register native Tab completion for Mood Shell commands, options, layouts, and built-in moods in Zsh, Bash, and PowerShell; Zsh and Bash also complete theme file paths.
- Initialize Zsh's standard completion system when needed, restoring command and path completion for users who removed Oh My Zsh.
- Generate completion definitions from the CLI at shell startup; repeated initialization remains safe and prompt hooks retain their behavior.
- Keep inline history suggestions outside this release's scope.

## 0.3.0 — 2026-10-02

- Create shareable Markdown themes with `theme new`, starting from any built-in mood.
- Export your current colors and layout with `theme export`, with an optional new name.
- Preview a local theme with `theme preview --file` and explicitly save it with `theme apply`.
- Read exactly one fenced `moodsh` block as validated TOML; prose and other code blocks are never executed.
- Reject ambiguous, malformed, oversized, and invalid theme documents. Creating or exporting a theme never overwrites an existing file.
- Include an editable Afterhours theme and format guide in release archives.
- Keep runtime TOML, shell hooks, and prompt rendering compatible. No additional dependencies.

## 0.2.0 — 2026-10-02

- Edit all four prompt colors inside `moodsh customize`, with live success and failure previews.
- Type or paste hex colors, discard individual edits, and keep drafts while browsing moods. Save only when ready; Esc or Ctrl-C cancels without writing.
- Keep incomplete colors out of prompt rendering and validate pasted text without interpreting it as keystrokes.
- Label saved custom palettes, explain each color's role, and pause the picker when a terminal is too small.
- Document installation without Rust on macOS, Linux, and Windows, and explain how to fix `cargo: command not found` when Rust is already installed.
- Preserve existing config format, prompt rendering, and shell hooks. No additional dependencies.

## 0.1.0 — 2026-10-02

- Add five built-in moods: dusk, aurora, ember, ocean, and paper.
- Add interactive theme previews, compact and two-line layouts, and custom colors.
- Save validated TOML configuration with atomic file replacement.
- Add prompt hooks for Zsh, Bash, and PowerShell 7.
- Show the previous command's failure status and preserve existing Zsh/Bash hooks.
- Add config inspection, environment diagnostics, and plain-color output.
- Add macOS, Linux, and Windows CI and tagged binary releases.
