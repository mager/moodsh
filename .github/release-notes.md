Mood Shell 0.4.0 adds Tab completion to Zsh, Bash, and PowerShell without changing the prompt you already use.

- Complete `moodsh` subcommands, flags, layouts, and built-in moods with Tab. Zsh and Bash also complete theme file paths. For example, type `moodsh theme set em` and press Tab to get `ember`.
- In Zsh, Mood Shell starts the standard completion system if another framework has not already done so. Normal command and path completion works after removing Oh My Zsh.
- Bash and PowerShell keep their existing command and path completion. Completion definitions are generated from Mood Shell's CLI at shell startup and stay current with the installed version.

This release is for Tab completion. Inline suggestions from command history are not included. The five moods, live picker, Markdown themes, runtime config, and prompt hooks remain compatible.

Every archive includes the Afterhours theme and format guide in `themes/`. Download the archive for your platform and follow the README to put `moodsh` (or `moodsh.exe`) on PATH. Rust and Cargo are not required. SHA-256 checksums accompany every archive. Reopen your shell after upgrading to load the new completion definitions.

This is an early public release for prompt personalization on Zsh, Bash, and PowerShell 7, with native macOS, Linux, and Windows builds. Terminal backgrounds, cursor animations, syntax highlighting, inline history suggestions, and Fish support are not implemented. Shell startup files are never edited automatically.
