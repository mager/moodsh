//! Zsh history suggestions are bundled at build time, never downloaded at shell startup.
pub const ZSH: &str = concat!(
    r#"
# Moodsh inline history suggestions. Keep an already-loaded implementation intact.
if [[ -o interactive && -z ${NO_COLOR+x} && ${TERM-} != dumb ]] &&
   (( ! ${+functions[_zsh_autosuggest_start]} )); then
  # Avoid suggestion work for large pasted buffers; respect explicit user settings.
  (( ${+ZSH_AUTOSUGGEST_BUFFER_MAX_SIZE} )) || typeset -g ZSH_AUTOSUGGEST_BUFFER_MAX_SIZE=256
"#,
    include_str!("../vendor/zsh-autosuggestions/zsh-autosuggestions.zsh"),
    "\nfi\n"
);
