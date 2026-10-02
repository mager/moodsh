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

#[test]
fn markdown_theme_create_preview_apply_export_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    let file = dir.path().join("afterhours.md");
    let file_arg = file.to_str().unwrap();
    let out = run(&config, &["theme", "new", file_arg, "--from", "ocean"]);
    assert!(out.status.success(), "{out:?}");
    assert!(!config.exists());
    let content = std::fs::read_to_string(&file).unwrap();
    let edited = content
        .replace("#7DCFFF", "#FF77CC")
        .replace("compact", "two-line");
    std::fs::write(&file, &edited).unwrap();
    let preview = run(&config, &["theme", "preview", "--file", file_arg]);
    assert!(preview.status.success(), "{preview:?}");
    assert!(
        String::from_utf8(preview.stdout)
            .unwrap()
            .contains("afterhours ~/code/moodsh")
    );
    assert!(!config.exists(), "Preview must not write runtime config");
    assert!(
        !run(&config, &["theme", "preview", "dusk", "--file", file_arg])
            .status
            .success()
    );
    assert!(run(&config, &["theme", "apply", file_arg]).status.success());
    assert_eq!(std::fs::read_to_string(&file).unwrap(), edited);
    let saved = std::fs::read_to_string(&config).unwrap();
    assert!(saved.contains("#FF77CC"));
    assert!(saved.contains("two-line"));
    let prompt = run(&config, &["prompt", "--status", "7"]);
    assert!(
        String::from_utf8(prompt.stdout)
            .unwrap()
            .starts_with("afterhours ")
    );
    let exported = dir.path().join("shared theme.md");
    let exported_arg = exported.to_str().unwrap();
    assert!(
        run(&config, &["theme", "export", exported_arg])
            .status
            .success()
    );
    assert_eq!(std::fs::read_to_string(&config).unwrap(), saved);
    assert!(
        run(&config, &["theme", "apply", exported_arg])
            .status
            .success()
    );
    assert_eq!(std::fs::read_to_string(&config).unwrap(), saved);
    for args in [
        vec!["theme", "new", file_arg],
        vec!["theme", "export", file_arg],
    ] {
        assert!(!run(&config, &args).status.success());
        assert_eq!(std::fs::read_to_string(&file).unwrap(), edited);
    }
}

#[test]
fn invalid_theme_files_and_existing_config_are_never_clobbered() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    let file = dir.path().join("theme.md");
    let file_arg = file.to_str().unwrap();
    assert!(run(&config, &["theme", "set", "ember"]).status.success());
    let saved = std::fs::read_to_string(&config).unwrap();
    for document in [
        "# Just prose",
        "```moodsh\n[mood]\nname = 'broken'\n```",
        "```moodsh\n",
    ] {
        std::fs::write(&file, document).unwrap();
        for command in ["apply", "preview"] {
            let args = if command == "apply" {
                vec!["theme", command, file_arg]
            } else {
                vec!["theme", command, "--file", file_arg]
            };
            let out = run(&config, &args);
            assert!(!out.status.success());
            assert!(
                String::from_utf8(out.stderr)
                    .unwrap()
                    .contains("Invalid theme file")
            );
            assert_eq!(std::fs::read_to_string(&config).unwrap(), saved);
        }
    }
    std::fs::remove_file(&file).unwrap();
    assert!(
        run(&config, &["theme", "export", file_arg, "--name", "my-mood"])
            .status
            .success()
    );
    std::fs::write(&config, "broken = [").unwrap();
    assert!(!run(&config, &["theme", "apply", file_arg]).status.success());
    assert_eq!(std::fs::read_to_string(&config).unwrap(), "broken = [");
    // A valid document can still be previewed while repairing a broken runtime config.
    assert!(
        run(&config, &["theme", "preview", "--file", file_arg])
            .status
            .success()
    );
}

#[test]
fn markdown_documents_are_data_and_names_are_validated() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    let file = dir.path().join("theme with spaces.md");
    let file_arg = file.to_str().unwrap();
    assert!(!run(&config, &["theme", "new", file_arg]).status.success());
    assert!(!file.exists());
    assert!(
        run(&config, &["theme", "new", file_arg, "--name", "safe-name"])
            .status
            .success()
    );
    let marker = dir.path().join("OWNED");
    let mut document = std::fs::read_to_string(&file).unwrap();
    document.push_str(&format!("\n```sh\ntouch '{}'\n```\n", marker.display()));
    std::fs::write(&file, document).unwrap();
    assert!(run(&config, &["theme", "apply", file_arg]).status.success());
    assert!(!marker.exists());
    let export = dir.path().join("invalid.md");
    assert!(
        !run(
            &config,
            &[
                "theme",
                "export",
                export.to_str().unwrap(),
                "--name",
                "$(whoami)"
            ]
        )
        .status
        .success()
    );
    assert!(!export.exists());
}

#[test]
fn untrusted_theme_errors_cannot_inject_terminal_controls() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    let file = dir.path().join("theme.md");
    let file_arg = file.to_str().unwrap();
    assert!(run(&config, &["theme", "new", file_arg]).status.success());
    let document = std::fs::read_to_string(&file).unwrap();
    std::fs::write(
        &file,
        document.replace("#C4A7E7", "\\u001b]52;c;payload\\u0007\\u202e"),
    )
    .unwrap();
    let out = run(&config, &["theme", "apply", file_arg]);
    assert!(!out.status.success());
    let error = String::from_utf8(out.stderr).unwrap();
    assert!(error.contains("Invalid color"));
    assert!(!error.contains(['\x1b', '\x07', '\u{202e}']));
    assert!(!config.exists());
}
