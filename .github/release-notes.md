Mood Shell 0.3.0 introduces shareable Markdown themes: a palette, a layout, and the explanation behind them in one readable file.

- `moodsh theme new afterhours.md --from ocean` creates an editable theme.
- `moodsh theme export my-mood.md --name my-mood` shares the colors you saved in the live picker.
- `moodsh theme preview --file afterhours.md` lets you inspect a local theme before saving.
- `moodsh theme apply afterhours.md` validates and applies it to your next prompt.

Theme documents contain exactly one fenced `moodsh` block of TOML. Prose and other code blocks are never executed. New/export refuse to overwrite files; malformed themes leave the active config intact. The existing runtime config and shell hooks are unchanged. No new dependencies.

Every archive includes the Afterhours theme and format guide in `themes/`. Download the archive for your platform and follow the README to put `moodsh` (or `moodsh.exe`) on PATH. Rust and Cargo are not required. SHA-256 checksums accompany every archive.

This is an early public release for prompt personalization on Zsh, Bash, and PowerShell 7, with native macOS, Linux, and Windows builds. Terminal backgrounds, cursor animations, syntax highlighting, autosuggestions, and Fish support are not implemented. Shell startup files are never edited automatically.
