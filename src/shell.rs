use crate::prompt::Shell;
use anyhow::{Result, bail};
use std::path::Path;

pub fn init(shell: Shell, executable: &Path) -> Result<String> {
    let path = executable
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Executable path is not valid Unicode"))?;
    if path.chars().any(char::is_control) {
        bail!("Executable path contains control characters");
    }
    let posix = format!("'{}'", path.replace('\'', "'\\''"));
    let powershell = format!("'{}'", path.replace('\'', "''"));
    let script = match shell {
        Shell::Zsh => r#"# Mood Shell: literal prompt text; do not evaluate directory names as shell code.
setopt PROMPT_PERCENT
unsetopt PROMPT_SUBST
_moodsh_precmd() {
  local moodsh_status=$?
  local moodsh_rendered
  if moodsh_rendered=$(@EXE@ prompt --shell zsh --status "$moodsh_status"); then
    PROMPT="$moodsh_rendered"
  else
    PROMPT='%~ > '
  fi
  # Zsh stops running remaining precmd hooks on a nonzero return.
  return 0
}
typeset -ga precmd_functions
precmd_functions=(_moodsh_precmd ${precmd_functions:#_moodsh_precmd})
# Enable Zsh's standard Tab completion when no other framework has done so.
if (( ! $+functions[compdef] )); then
  autoload -Uz compinit
  compinit -i
fi
"#.replace("@EXE@", &posix),
        Shell::Bash => r#"_moodsh_prompt() {
  local moodsh_status=$?
  local moodsh_rendered
  if moodsh_rendered=$(@EXE@ prompt --shell bash --status "$moodsh_status"); then
    PS1="$moodsh_rendered"
  else
    PS1='\w > '
  fi
  return "$moodsh_status"
}
# Preserve existing hooks, including array hooks on newer Bash versions.
case "$(declare -p PROMPT_COMMAND 2>/dev/null)" in
  'declare -a'*)
    _moodsh_hooks=()
    for _moodsh_hook in "${PROMPT_COMMAND[@]}"; do
      [[ "$_moodsh_hook" == _moodsh_prompt ]] || _moodsh_hooks+=("$_moodsh_hook")
    done
    PROMPT_COMMAND=(_moodsh_prompt "${_moodsh_hooks[@]}")
    unset _moodsh_hooks _moodsh_hook
    ;;
  *)
    case ";${PROMPT_COMMAND-};" in
      *';_moodsh_prompt;'*) ;;
      *) PROMPT_COMMAND="_moodsh_prompt${PROMPT_COMMAND:+; $PROMPT_COMMAND}" ;;
    esac
    ;;
esac
"#.replace("@EXE@", &posix),
        Shell::Powershell => r#"function global:prompt {
    $moodshOK = $?
    $moodshNative = $global:LASTEXITCODE
    $moodshStatus = if ($moodshOK) { 0 } elseif ($moodshNative -is [int] -and $moodshNative -ne 0) { $moodshNative } else { 1 }
    $moodshRendered = & @EXE@ prompt --shell powershell --status $moodshStatus
    $moodshFailed = $LASTEXITCODE -ne 0
    $global:LASTEXITCODE = $moodshNative
    if ($moodshFailed) { return "$($executionContext.SessionState.Path.CurrentLocation)> " }
    return ($moodshRendered -join "`n")
}
"#.replace("@EXE@", &powershell),
        Shell::Plain => bail!("Choose a shell: zsh, bash, or powershell"),
    };
    Ok(script)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn executable_paths_are_quoted() {
        let path = Path::new("/tmp/Mager's tools/moodsh");
        assert!(init(Shell::Zsh, path).unwrap().contains("Mager'\\''s"));
        assert!(init(Shell::Bash, path).unwrap().contains("Mager'\\''s"));
        assert!(init(Shell::Powershell, path).unwrap().contains("Mager''s"));
    }
}
