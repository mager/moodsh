//! Portable theme documents. Markdown prose is ignored; only one labeled block is data.
use crate::theme::Mood;
use serde::{Deserialize, Serialize};

// Portable themes deliberately exclude machine-local directory preferences.
#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ThemeDocument {
    mood: Mood,
}
use anyhow::{Context, Result, bail};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const MAX_BYTES: u64 = 64 * 1024;

struct Fence {
    marker: u8,
    length: usize,
    theme: bool,
}

// This intentionally supports only standalone fenced blocks, not a Markdown renderer.
// Tracking other fences prevents an example inside a larger code block becoming a theme.
fn fence_line(line: &str) -> Option<(u8, usize, &str)> {
    let spaces = line.bytes().take_while(|b| *b == b' ').count();
    if spaces > 3 {
        return None;
    }
    let line = &line[spaces..];
    let marker = *line.as_bytes().first()?;
    if marker != b'`' && marker != b'~' {
        return None;
    }
    let length = line.bytes().take_while(|b| *b == marker).count();
    (length >= 3).then(|| (marker, length, line[length..].trim()))
}

fn parse(document: &str) -> Result<Mood> {
    let mut fence: Option<Fence> = None;
    let mut found = false;
    let mut data = String::new();
    for line in document.trim_start_matches('\u{feff}').lines() {
        if let Some(open) = &fence {
            if let Some((marker, length, info)) = fence_line(line)
                && marker == open.marker
                && length >= open.length
                && info.is_empty()
            {
                fence = None;
            } else if open.theme {
                data.push_str(line);
                data.push('\n');
            }
        } else if let Some((marker, length, info)) = fence_line(line) {
            if marker == b'`' && info.contains('`') {
                continue;
            }
            let theme = info == "moodsh";
            if theme {
                if found {
                    bail!(
                        "A theme file must contain exactly one fenced moodsh block; found another"
                    );
                }
                found = true;
            }
            fence = Some(Fence {
                marker,
                length,
                theme,
            });
        }
    }
    if fence.is_some_and(|open| open.theme) {
        bail!("The moodsh block is not closed; add a matching closing fence");
    }
    if !found {
        bail!(
            "No theme found. Add a fenced moodsh block, or start with `moodsh theme new my-mood.md`"
        );
    }
    let config: ThemeDocument =
        toml::from_str(&data).context("Invalid TOML in the moodsh block")?;
    config.mood.validate()?;
    Ok(config.mood)
}

pub fn load(path: &Path) -> Result<Mood> {
    let metadata = fs::metadata(path).with_context(|| format!("Cannot read {}", path.display()))?;
    if !metadata.is_file() {
        bail!("Theme path must be a regular file: {}", path.display());
    }
    let mut document = String::new();
    File::open(path)?
        .take(MAX_BYTES + 1)
        .read_to_string(&mut document)
        .with_context(|| format!("Cannot read {} as UTF-8 Markdown", path.display()))?;
    if document.len() as u64 > MAX_BYTES {
        bail!("Theme files must be at most 64 KiB: {}", path.display());
    }
    parse(&document).with_context(|| format!("Invalid theme file at {}", path.display()))
}

pub fn write(path: &Path, mood: &Mood) -> Result<()> {
    mood.validate()?;
    if !matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("md" | "markdown")
    ) {
        bail!("Use a .md or .markdown filename for a theme document");
    }
    let data = toml::to_string_pretty(&ThemeDocument { mood: mood.clone() })?;
    let document = format!(
        "# {name}\n\nA Mood Shell prompt theme. Add your inspiration, author credit, and preferred\nterminal background here. Only the fenced moodsh block below is read by Mood Shell.\n\n```moodsh\n{data}```\n\n## Color roles\n\n- `accent`: prompt arrow after a successful command.\n- `path`: directory text.\n- `muted`: mood name in the two-line layout.\n- `error`: failed-command status and arrow.\n\nLayout is `compact` or `two-line`. Colors use `#RRGGBB`.\nThis theme changes the prompt; it does not change your terminal background.\n\nPreview with `moodsh theme preview --file <this-file.md>`.\nApply with `moodsh theme apply <this-file.md>`.\n",
        name = mood.name,
    );
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut file = tempfile::NamedTempFile::new_in(parent)
        .with_context(|| format!("Cannot create a theme in {}", parent.display()))?;
    file.write_all(document.as_bytes())?;
    file.as_file().sync_all()?;
    file.persist_noclobber(path).with_context(|| {
        format!(
            "Cannot create {}; existing files are never overwritten",
            path.display()
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block() -> String {
        format!(
            "```moodsh\n{}```\n",
            toml::to_string(&ThemeDocument::default()).unwrap()
        )
    }

    #[test]
    fn reads_only_the_labeled_block_and_handles_common_line_endings() {
        let document = format!(
            "# My mood\n\n$(touch OWNED)\n\n{}\n```sh\nexit 1\n```\n",
            block()
        );
        assert_eq!(parse(&document).unwrap(), Mood::default());
        assert_eq!(
            parse(&format!("\u{feff}{}", document.replace('\n', "\r\n"))).unwrap(),
            Mood::default()
        );
        assert_eq!(
            parse(&block().replace("```", "~~~~")).unwrap(),
            Mood::default()
        );
        assert_eq!(
            parse(&block().replace("```", "   ````")).unwrap(),
            Mood::default()
        );
    }

    #[test]
    fn ignores_nested_examples_and_rejects_ambiguous_or_unclosed_blocks() {
        let nested = format!("````markdown\n{}\n````\n", block());
        assert!(parse(&nested).is_err());
        assert_eq!(parse(&(nested + &block())).unwrap(), Mood::default());
        assert!(parse(&(block() + &block())).is_err());
        assert!(parse(block().trim_end().trim_end_matches('`')).is_err());
        assert!(parse(&block().replace("```moodsh", "````moodsh")).is_err());
        assert!(parse(&block().replace("```moodsh", "    ```moodsh")).is_err());
        assert!(parse(&block().replace("```moodsh", "```toml")).is_err());
    }

    #[test]
    fn rejects_unknown_fields_and_prompt_injection() {
        for data in [
            block().replace("name = \"dusk\"", "name = \"$(touch OWNED)\""),
            block().replace("#C4A7E7", "\\u001b[31m"),
            block().replace("name = \"dusk\"", "name = \"dusk\"\ncommand = \"echo hi\""),
            block().replace("compact", "animated"),
            block().replace("```\n", "[path]\ncolor = \"terminal\"\n```\n"),
        ] {
            assert!(parse(&data).is_err(), "{data}");
        }
    }

    #[test]
    fn export_roundtrips_and_never_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("my mood.md");
        let mut mood = Mood::builtin("paper").unwrap();
        mood.name = "my-mood".into();
        write(&path, &mood).unwrap();
        assert_eq!(load(&path).unwrap(), mood);
        assert!(write(&path, &Mood::default()).is_err());
        assert_eq!(load(&path).unwrap(), mood);
        assert!(write(&dir.path().join("config.toml"), &mood).is_err());
    }

    #[test]
    fn rejects_oversized_and_non_utf8_files_and_directories() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load(dir.path()).is_err());
        let path = dir.path().join("theme.md");
        fs::write(&path, vec![b' '; MAX_BYTES as usize + 1]).unwrap();
        assert!(load(&path).unwrap_err().to_string().contains("64 KiB"));
        fs::write(&path, [0xff, 0xfe]).unwrap();
        assert!(load(&path).is_err());
    }
}
