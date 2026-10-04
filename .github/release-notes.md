Mood Shell 0.7.0 adds **directory display controls** on Zsh, Bash, and PowerShell 7.

The path already appears in both layouts. If a palette makes it hard to see, run:

```sh
moodsh path --color terminal
```

The directory now uses your terminal's configured foreground. Choose `moodsh path --format full` for the full directory, or `--format home` for `~/…`. Restore palette coloring with `--color theme`. Preferences survive theme changes and stay out of exported Markdown themes.

The picker has a live **P** toggle for path color and labels Paper for light terminals. Applying a path hex color with **2** switches back to theme coloring. Enter saves, Esc cancels.

Existing configs retain their current appearance. Shell hooks, safe prompt rendering, no-color output, Git shortcuts, and Tab completion remain compatible. No new dependencies. New settings appear on the next prompt; reopen your shell to refresh command completion after upgrading.

Ready-to-run binaries cover Apple Silicon and Intel macOS, x64 Linux, and x64 Windows, with SHA-256 checksums. Rust is not required.
