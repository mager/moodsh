Mood Shell 0.5.0 adds **Git shortcuts**: 25 familiar names from the Oh My Zsh Git vocabulary, implemented natively for Zsh, Bash, and PowerShell 7.

Run `moodsh shortcuts git` to inspect every expansion. To enable them, add `--shortcuts git` to your existing `moodsh init` line and open a new shell. Try `gst`, `gaa`, `gcmsg "message"`, `gco branch`, or `glog`.

Existing command names always win. PowerShell normally keeps its built-in `gc`, `gl`, and `gp`. PowerShell shortcuts raise a terminating error on Git failure and preserve `$LASTEXITCODE`; scripts that handle Git exit codes themselves should invoke `git` directly. Git is a separate prerequisite. This is a focused shortcut set, not an arbitrary shell-code loader; shortcuts stay out of Markdown themes and add no per-prompt work. Remove the flag and start a new shell to disable them.

Prompt colors, layouts, Markdown themes, Tab completion, config, and shell hooks remain compatible. No new Rust dependencies. No force-push or hard-reset shortcuts are included.

Download the archive for your platform; Rust is not required. Every archive includes checksums, documentation, and an Afterhours example theme. Terminal backgrounds, inline history suggestions, syntax highlighting, animations, and Fish support remain outside this release.
