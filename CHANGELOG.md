# Changelog

Versions follow Semantic Versioning. Before 1.0, breaking changes increment the minor version; compatible fixes increment the patch version. Git tags use `vMAJOR.MINOR.PATCH` and must match `Cargo.toml`.

## 0.1.0 — 2026-10-02

- Add five built-in moods: dusk, aurora, ember, ocean, and paper.
- Add interactive theme previews, compact and two-line layouts, and custom colors.
- Save validated TOML configuration with atomic file replacement.
- Add prompt hooks for Zsh, Bash, and PowerShell 7.
- Show the previous command's failure status and preserve existing Zsh/Bash hooks.
- Add config inspection, environment diagnostics, and plain-color output.
- Add macOS, Linux, and Windows CI and tagged binary releases.
