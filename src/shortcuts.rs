use crate::prompt::Shell;
use anyhow::{Context, Result, bail};
use std::{path::Path, process::Command};

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

pub fn init(shell: Shell, executable: &Path) -> Result<String> {
    let mut script = String::from("\n# moodsh Git shortcuts: existing command names always win.\n");
    for (name, command) in GIT {
        match shell {
            Shell::Zsh | Shell::Bash => script.push_str(&format!(
                "if ! command -v {name} >/dev/null 2>&1; then alias {name}='{command}'; fi\n"
            )),
            Shell::Powershell => script.push_str(&format!(
                "if (-not (Get-Command '{name}' -ErrorAction SilentlyContinue)) {{ function global:{name} {{ & {command} @args; if ($LASTEXITCODE -ne 0) {{ throw \"Git exited with status $LASTEXITCODE\" }} }} }}\n"
            )),
            Shell::Plain => bail!("Choose a shell: zsh, bash, or powershell"),
        }
    }
    let exe = crate::shell::quote_executable(shell, executable)?;
    script.push_str(&match shell {
        Shell::Zsh | Shell::Bash => format!(
            "if ! command -v gprom >/dev/null 2>&1; then\n  function gprom {{\n    local moodsh_branch\n    moodsh_branch=$({exe} __git-main-branch) || return $?\n    git pull --rebase origin \"$moodsh_branch\" \"$@\"\n  }}\nfi\n"
        ),
        Shell::Powershell => format!(
            "if (-not (Get-Command 'gprom' -ErrorAction SilentlyContinue)) {{\n  function global:gprom {{\n    $moodshBranch = & {exe} __git-main-branch\n    if ($LASTEXITCODE -ne 0) {{ throw \"Cannot resolve origin's main branch\" }}\n    & git pull --rebase origin $moodshBranch @args\n    if ($LASTEXITCODE -ne 0) {{ throw \"Git exited with status $LASTEXITCODE\" }}\n  }}\n}}\n"
        ),
        Shell::Plain => bail!("Choose a shell: zsh, bash, or powershell"),
    });
    Ok(script)
}

fn git_output(args: &[&str]) -> Result<Option<String>> {
    let output = Command::new("git")
        .args(args)
        .output()
        .context("Cannot run Git; install Git to use gprom")?;
    if !output.status.success() {
        return Ok(None);
    }
    Ok(Some(
        String::from_utf8(output.stdout)
            .context("Git returned a non-Unicode branch name")?
            .trim_end_matches('\n')
            .trim_end_matches('\r')
            .to_owned(),
    ))
}

/// Resolve only local Git metadata. Never fetch, change directories, or pull here.
pub fn git_main_branch() -> Result<String> {
    if git_output(&["rev-parse", "--git-dir"])?.is_none() {
        bail!("gprom must be run inside a Git repository");
    }
    if git_output(&["remote", "get-url", "origin"])?.is_none() {
        bail!("gprom requires an origin remote");
    }
    if let Some(reference) = git_output(&["symbolic-ref", "--quiet", "refs/remotes/origin/HEAD"])?
        && let Some(branch) = reference.strip_prefix("refs/remotes/origin/")
        && git_output(&["show-ref", "--verify", "--quiet", &reference])?.is_some()
        && git_output(&["check-ref-format", "--branch", branch])?.is_some()
    {
        return Ok(branch.to_owned());
    }
    for prefix in ["refs/remotes/origin", "refs/heads"] {
        for branch in ["main", "trunk", "mainline", "default", "stable", "master"] {
            let reference = format!("{prefix}/{branch}");
            if git_output(&["show-ref", "--verify", "--quiet", &reference])?.is_some() {
                return Ok(branch.to_owned());
            }
        }
    }
    bail!(
        "Cannot resolve origin's main branch; fetch origin and set its default with git remote set-head origin <branch>"
    )
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
