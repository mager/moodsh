use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Palette {
    pub accent: String,
    pub path: String,
    pub muted: String,
    pub error: String,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Layout {
    #[default]
    Compact,
    TwoLine,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Mood {
    pub name: String,
    pub palette: Palette,
    pub layout: Layout,
}

pub const NAMES: [&str; 5] = ["dusk", "aurora", "ember", "ocean", "paper"];

impl Mood {
    pub fn builtin(name: &str) -> Result<Self> {
        let colors = match name {
            "dusk" => ["#C4A7E7", "#E0DEF4", "#908CAA", "#EB6F92"],
            "aurora" => ["#A6E3A1", "#89DCEB", "#9399B2", "#F38BA8"],
            "ember" => ["#FFB454", "#FFD6A5", "#B39B8D", "#FF6B6B"],
            "ocean" => ["#7DCFFF", "#C0CAF5", "#8992B0", "#F7768E"],
            "paper" => ["#5C3599", "#263238", "#62676B", "#B42338"],
            _ => bail!("Unknown mood '{name}'. Try: {}", NAMES.join(", ")),
        };
        Ok(Self {
            name: name.into(),
            palette: Palette {
                accent: colors[0].into(),
                path: colors[1].into(),
                muted: colors[2].into(),
                error: colors[3].into(),
            },
            layout: Layout::Compact,
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty()
            || self.name.len() > 48
            || !self
                .name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        {
            bail!("Mood name must be 1–48 ASCII letters, digits, hyphens, or underscores");
        }
        for color in [
            &self.palette.accent,
            &self.palette.path,
            &self.palette.muted,
            &self.palette.error,
        ] {
            rgb(color)?;
        }
        Ok(())
    }
}

impl Default for Mood {
    fn default() -> Self {
        Self::builtin("dusk").expect("built-in mood")
    }
}

pub fn rgb(value: &str) -> Result<(u8, u8, u8)> {
    let bytes = value.as_bytes();
    if bytes.len() != 7 || bytes[0] != b'#' || !bytes[1..].iter().all(u8::is_ascii_hexdigit) {
        bail!("Invalid color '{value}': use #RRGGBB, for example #C4A7E7");
    }
    Ok((
        u8::from_str_radix(&value[1..3], 16)?,
        u8::from_str_radix(&value[3..5], 16)?,
        u8::from_str_radix(&value[5..7], 16)?,
    ))
}
