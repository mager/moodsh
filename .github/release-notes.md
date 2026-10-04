Mood Shell 0.7.1 makes **the terminal’s normal text color the default for directory paths**.

New installations and configs without a path-color preference use `terminal`. Existing explicit choices are preserved. Both prompt layouts show the directory, with `~/…` still the default format.

- `moodsh path --color theme`: use your theme’s path color.
- `moodsh path --color terminal`: use the default terminal text color.
- `moodsh path --format full`: show the full directory.

An explicit `theme set --path '#RRGGBB'` now enables that color, matching the picker. Named/file previews still show the portable palette. No shell-hook changes or new dependencies.

Upgrade the binary and open a new terminal to refresh completion. Your saved palette, layout, path format, and Git shortcuts remain available. Downloads include macOS, Linux, and Windows binaries with SHA-256 checksums.
