use crate::prompt::Shell;
use anyhow::{Result, bail};

// Familiar command mappings from Oh My Zsh's Git vocabulary. No external shell
// code is loaded: these fixed tokens are the complete, auditable shortcut set.
pub const GIT: &[(&str, &str)] = &[
    ("g", "git"),
    ("ga", "git add"),
    ("gaa", "git add --all"),
    ("gap", "git apply"),
    ("gb", "git branch"),
    ("gba", "git branch --all"),
    ("gc", "git commit --verbose"),
    ("gcmsg", "git commit --message"),
    ("gco", "git checkout"),
    ("gcb", "git checkout -b"),
    ("gd", "git diff"),
    ("gds", "git diff --staged"),
    ("gf", "git fetch"),
    ("gl", "git pull"),
    ("gp", "git push"),
    ("glog", "git log --oneline --decorate --graph"),
    ("gloga", "git log --oneline --decorate --graph --all"),
    ("gst", "git status"),
    ("gss", "git status --short"),
    ("gsb", "git status --short --branch"),
    ("gsta", "git stash push"),
    ("gstl", "git stash list"),
    ("gstp", "git stash pop"),
    ("gsw", "git switch"),
    ("gswc", "git switch --create"),
];

pub fn init(shell: Shell) -> Result<String> {
    let mut script = String::from("\n# moodsh Git shortcuts: existing command names always win.\n");
    for (name, command) in GIT {
        match shell {
            Shell::Zsh | Shell::Bash => script.push_str(&format!(
                "if ! command -v {name} >/dev/null 2>&1; then alias {name}='{command}'; fi\n"
            )),
            Shell::Powershell => script.push_str(&format!(
                "if (-not (Get-Command '{name}' -ErrorAction SilentlyContinue)) {{ function global:{name} {{ & {command} @args }} }}\n"
            )),
            Shell::Plain => bail!("Choose a shell: zsh, bash, or powershell"),
        }
    }
    Ok(script)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_is_unique_and_contains_only_literal_command_tokens() {
        let mut names = std::collections::HashSet::new();
        for (name, command) in GIT {
            assert!(names.insert(name));
            assert!(name.chars().all(|c| c.is_ascii_lowercase()));
            assert!(command.starts_with("git"));
            assert!(
                command
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || " -".contains(c))
            );
        }
    }
}
