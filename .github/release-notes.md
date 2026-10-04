Mood Shell 0.8.0 adds **faint history suggestions while you type in Zsh**.

Run `echo moodsh-ready`, then type `echo moodsh-` at the next prompt. The matching suffix appears in gray. Press **Right Arrow** at the end of your input to accept it, then **Enter** to run it. Tab still completes commands and files.

Upgrade the binary and **open a new terminal**. Your existing `moodsh init zsh` line enables suggestions, including when you use `--shortcuts git`. Add `--no-suggestions` to that line and start a new shell to opt out.

- Local command history only; no matching history means no suggestion.
- Available in Zsh on macOS/Linux. Bash and PowerShell input behavior is unchanged.
- Existing suggestion engines, keybindings, and history persistence settings are preserved.
- Bundles the MIT-licensed zsh-autosuggestions v0.7.1 engine. No extra installation, network calls, or Rust calls per keystroke.
- Real Zsh terminal tests cover typing, acceptance without execution, cancellation, Unicode, custom bindings, and Tab completion together.

Prompt colors, path preferences, Markdown themes, and Git shortcuts remain compatible. Downloads include macOS, Linux, and Windows binaries with SHA-256 checksums and third-party notices. Terminal backgrounds, cursor effects, and syntax highlighting remain outside Mood Shell's scope.
