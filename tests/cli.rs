use std::{
    path::Path,
    process::{Command, Output},
};

fn run(path: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_moodsh"))
        .env("MOODSH_CONFIG", path)
        .env("NO_COLOR", "1")
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn version_is_semver() {
    let dir = tempfile::tempdir().unwrap();
    let out = run(&dir.path().join("config.toml"), &["--version"]);
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap().trim(),
        format!("moodsh {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn theme_changes_survive_new_processes_and_bad_inputs_preserve_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let out = run(
        &path,
        &[
            "theme", "set", "aurora", "--accent", "#123abc", "--layout", "two-line",
        ],
    );
    assert!(out.status.success(), "{:?}", out);
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("#123abc"));
    let out = run(&path, &["prompt", "--status", "42"]);
    assert!(out.status.success());
    let prompt = String::from_utf8(out.stdout).unwrap();
    assert!(prompt.starts_with("aurora "));
    assert!(prompt.contains("[42]\n> "));
    assert!(!prompt.contains('\x1b'));
    for args in [
        vec!["theme", "set", "missing"],
        vec!["theme", "set", "dusk", "--accent", "oops"],
    ] {
        assert!(!run(&path, &args).status.success());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), content);
    }
}

#[test]
fn config_errors_are_actionable_and_never_clobbered() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "bad = [").unwrap();
    for args in [vec!["doctor"], vec!["theme", "set", "ocean"]] {
        let out = run(&path, &args);
        assert!(!out.status.success());
        assert!(
            String::from_utf8(out.stderr)
                .unwrap()
                .contains("Invalid config")
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "bad = [");
    }
}

#[test]
fn init_and_preview_do_not_write_configuration() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    for shell in ["bash", "zsh", "powershell"] {
        let out = run(&path, &["init", shell]);
        assert!(out.status.success());
        assert!(
            String::from_utf8(out.stdout)
                .unwrap()
                .contains("prompt --shell")
        );
    }
    assert!(run(&path, &["theme", "preview"]).status.success());
    assert!(!path.exists());
    assert!(!run(&path, &["customize"]).status.success());
}
