# Bundled Zsh history suggestion engine

Unmodified `zsh-autosuggestions.zsh` from zsh-users/zsh-autosuggestions v0.7.1,
commit `e52ee8ca55bcc56a17c828767a3f98f22a68d4eb`:
https://github.com/zsh-users/zsh-autosuggestions/tree/e52ee8ca55bcc56a17c828767a3f98f22a68d4eb

MIT licensed; see LICENSE and the copyright notice embedded in the script.
Moodsh embeds the script in its binary and emits it during `init zsh`.
The wrapper lives in `src/suggestions.rs`; no runtime download or extra install.
Upstream widget wrapping preserves existing keybindings, supports asynchronous
history lookup, and leaves execution to the user's normal Enter key.

To update: fetch the script and LICENSE from a reviewed upstream tag/commit,
update this provenance, and run the Rust and real-terminal suggestion tests.
