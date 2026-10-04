use crate::theme::Mood;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub mood: Mood,
    #[serde(default)]
    pub path: crate::prompt::PathSettings,
}

pub fn path() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("MOODSH_CONFIG") {
        if path.is_empty() {
            bail!("MOODSH_CONFIG must not be empty");
        }
        return Ok(path.into());
    }
    Ok(dirs::config_dir()
        .context("Cannot find your config directory; set MOODSH_CONFIG")?
        .join("moodsh/config.toml"))
}

pub fn load(path: &Path) -> Result<Config> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Config::default()),
        Err(err) => return Err(err).with_context(|| format!("Cannot read {}", path.display())),
    };
    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Invalid config at {}", path.display()))?;
    config.mood.validate()?;
    Ok(config)
}

pub fn save(path: &Path, config: &Config) -> Result<()> {
    config.mood.validate()?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent).with_context(|| format!("Cannot create {}", parent.display()))?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    write!(
        file,
        "# Mood Shell — https://github.com/mager/moodsh\n{}",
        toml::to_string_pretty(config)?
    )?;
    file.as_file().sync_all()?;
    file.persist(path)
        .with_context(|| format!("Cannot save {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_config_defaults_but_corrupt_config_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        assert_eq!(load(&path).unwrap(), Config::default());
        fs::write(&path, "not valid toml").unwrap();
        assert!(load(&path).is_err());
    }

    #[test]
    fn unspecified_path_color_uses_terminal_but_explicit_preferences_win() {
        use crate::prompt::{PathColor, PathFormat};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        assert_eq!(load(&path).unwrap().path.color, PathColor::Terminal);
        // A pre-0.7 config has only the mood table.
        let legacy = toml::to_string(&Config::default())
            .unwrap()
            .split("[path]")
            .next()
            .unwrap()
            .to_owned();
        fs::write(&path, &legacy).unwrap();
        assert_eq!(load(&path).unwrap().path.color, PathColor::Terminal);
        fs::write(&path, format!("{legacy}\n[path]\nformat = 'full'\n")).unwrap();
        let partial = load(&path).unwrap();
        assert_eq!(partial.path.color, PathColor::Terminal);
        assert_eq!(partial.path.format, PathFormat::Full);
        fs::write(&path, format!("{legacy}\n[path]\ncolor = 'theme'\n")).unwrap();
        assert_eq!(load(&path).unwrap().path.color, PathColor::Theme);
    }

    #[test]
    fn roundtrip_replace_and_reject_invalid_without_overwriting() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/config.toml");
        let mut config = Config::default();
        save(&path, &config).unwrap();
        config.mood = Mood::builtin("paper").unwrap();
        save(&path, &config).unwrap();
        assert_eq!(load(&path).unwrap(), config);
        config.mood.palette.accent = "bad".into();
        assert!(save(&path, &config).is_err());
        assert_eq!(load(&path).unwrap().mood.name, "paper");
    }
}
