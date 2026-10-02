use crate::theme::{Layout, Mood, rgb};
use std::path::Path;

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum Shell {
    Zsh,
    Bash,
    Powershell,
    Plain,
}

pub fn directory(cwd: &Path, home: Option<&Path>) -> String {
    let display = match home.and_then(|home| cwd.strip_prefix(home).ok()) {
        Some(relative) if relative.as_os_str().is_empty() => "~".into(),
        Some(relative) => format!("~/{}", relative.display()),
        None => cwd.display().to_string(),
    };
    // Directory names are untrusted terminal data. Remove control and bidi characters.
    display
        .chars()
        .filter(|c| {
            !c.is_control() && !matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        })
        .collect()
}

fn literal(text: &str, shell: Shell) -> String {
    match shell {
        Shell::Zsh => text.replace('%', "%%"),
        Shell::Bash => text
            .replace('\\', "\\\\")
            .replace('$', "\\$")
            .replace('`', "\\`"),
        Shell::Powershell | Shell::Plain => text.into(),
    }
}

fn invisible(text: &str, shell: Shell) -> String {
    match shell {
        Shell::Zsh => format!("%{{{text}%}}"),
        Shell::Bash => format!("\\[{text}\\]"),
        _ => text.into(),
    }
}

fn paint(text: &str, color: &str, shell: Shell, colors: bool) -> String {
    let text = literal(text, shell);
    if !colors {
        return text;
    }
    let (r, g, b) = rgb(color).expect("validated palette");
    format!(
        "{}{}{}",
        invisible(&format!("\x1b[38;2;{r};{g};{b}m"), shell),
        text,
        invisible("\x1b[0m", shell)
    )
}

pub fn render(mood: &Mood, cwd: &str, status: i32, shell: Shell, colors: bool) -> String {
    let path = paint(cwd, &mood.palette.path, shell, colors);
    let marker = paint(
        ">",
        if status == 0 {
            &mood.palette.accent
        } else {
            &mood.palette.error
        },
        shell,
        colors,
    );
    let failure = if status == 0 {
        String::new()
    } else {
        format!(
            " {}",
            paint(&format!("[{status}]"), &mood.palette.error, shell, colors)
        )
    };
    match mood.layout {
        Layout::Compact => format!("{path}{failure} {marker} "),
        Layout::TwoLine => format!(
            "{} {path}{failure}\n{marker} ",
            paint(&mood.name, &mood.palette.muted, shell, colors)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_shortening_obeys_path_boundaries() {
        assert_eq!(
            directory(Path::new("/home/me/code"), Some(Path::new("/home/me"))),
            "~/code"
        );
        assert_eq!(
            directory(Path::new("/home/merlin"), Some(Path::new("/home/me"))),
            "/home/merlin"
        );
    }

    #[test]
    fn removes_terminal_control_and_bidi_sequences() {
        assert_eq!(directory(Path::new("a\n\x1b\u{202e}b"), None), "ab");
    }

    #[test]
    fn escapes_prompt_syntax() {
        let mood = Mood::default();
        assert!(render(&mood, "%F{red}", 0, Shell::Zsh, false).starts_with("%%F{red}"));
        assert!(
            render(&mood, "$(touch bad)`date`\\n", 0, Shell::Bash, false)
                .starts_with("\\$(touch bad)\\`date\\`\\\\n")
        );
        let result = render(&mood, "demo", 7, Shell::Bash, true);
        assert!(result.contains("\\[\x1b["));
        assert!(result.contains("[7]"));
    }
}
