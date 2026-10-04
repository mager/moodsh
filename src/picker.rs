use crate::{
    config,
    prompt::{self, PathColor, PathSettings, Shell},
    theme::{Layout, Mood, NAMES, rgb},
};
use anyhow::{Context, Result, bail};
use crossterm::{
    cursor,
    event::{
        self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers,
    },
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{self, IsTerminal, Write},
    path::Path,
};

struct Screen;
impl Drop for Screen {
    fn drop(&mut self) {
        let _ = execute!(
            io::stdout(),
            DisableBracketedPaste,
            cursor::Show,
            LeaveAlternateScreen
        );
        let _ = terminal::disable_raw_mode();
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Field {
    Accent,
    Path,
    Muted,
    Error,
}

impl Field {
    const ALL: [Self; 4] = [Self::Accent, Self::Path, Self::Muted, Self::Error];

    fn label(self) -> &'static str {
        match self {
            Self::Accent => "accent (prompt arrow)",
            Self::Path => "path (directory)",
            Self::Muted => "muted (two-line mood name)",
            Self::Error => "error (failed command)",
        }
    }

    fn color(self, mood: &mut Mood) -> &mut String {
        match self {
            Self::Accent => &mut mood.palette.accent,
            Self::Path => &mut mood.palette.path,
            Self::Muted => &mut mood.palette.muted,
            Self::Error => &mut mood.palette.error,
        }
    }
}

struct Edit {
    field: Field,
    value: String,
    replace: bool,
    invalid_paste: bool,
}

struct Picker {
    choices: Vec<Mood>,
    selected: usize,
    layout: Layout,
    edit: Option<Edit>,
    path: PathSettings,
}

#[derive(Debug, PartialEq)]
enum Action {
    Continue,
    Save,
    Cancel,
}

impl Picker {
    fn new(current: Mood) -> Self {
        let mut choices = NAMES
            .iter()
            .map(|name| Mood::builtin(name).expect("built-in mood"))
            .collect::<Vec<_>>();
        // Keep the exact saved palette available, including custom color overrides.
        let selected = choices
            .iter()
            .position(|mood| mood == &current)
            .unwrap_or_else(|| {
                choices.push(current.clone());
                choices.len() - 1
            });
        Self {
            choices,
            selected,
            layout: current.layout,
            edit: None,
            path: PathSettings::default(),
        }
    }

    fn preview(&self) -> Mood {
        let mut mood = self.choices[self.selected].clone();
        mood.layout = self.layout;
        // Incomplete input never reaches prompt rendering or the config file.
        if let Some(edit) = &self.edit
            && rgb(&edit.value).is_ok()
        {
            *edit.field.color(&mut mood) = edit.value.clone();
        }
        mood
    }

    fn key(&mut self, key: KeyEvent) -> Action {
        if key.kind == KeyEventKind::Release {
            return Action::Continue;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Action::Cancel;
        }
        if let Some(edit) = &mut self.edit {
            match key.code {
                KeyCode::Esc => self.edit = None,
                KeyCode::Enter if !edit.invalid_paste && rgb(&edit.value).is_ok() => {
                    *edit.field.color(&mut self.choices[self.selected]) = edit.value.clone();
                    if edit.field == Field::Path {
                        self.path.color = PathColor::Theme;
                    }
                    self.edit = None;
                }
                KeyCode::Backspace => {
                    edit.value.pop();
                    edit.replace = false;
                    edit.invalid_paste = false;
                }
                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    edit.value.clear();
                    edit.replace = false;
                    edit.invalid_paste = false;
                }
                KeyCode::Char(c)
                    if !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                        && (c.is_ascii_hexdigit() || c == '#') =>
                {
                    edit.invalid_paste = false;
                    if edit.replace {
                        edit.value.clear();
                        edit.replace = false;
                    }
                    if edit.value.is_empty() {
                        edit.value.push('#');
                    }
                    if c.is_ascii_hexdigit() && edit.value.len() < 7 {
                        edit.value.push(c);
                    }
                }
                _ => (),
            }
            return Action::Continue;
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected = (self.selected + self.choices.len() - 1) % self.choices.len()
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.selected = (self.selected + 1) % self.choices.len()
            }
            KeyCode::Char('p' | 'P') => {
                self.path.color = if self.path.color == PathColor::Theme {
                    PathColor::Terminal
                } else {
                    PathColor::Theme
                };
            }
            KeyCode::Tab => {
                self.layout = if self.layout == Layout::Compact {
                    Layout::TwoLine
                } else {
                    Layout::Compact
                }
            }
            KeyCode::Char(c @ '1'..='4') => {
                let field = Field::ALL[(c as u8 - b'1') as usize];
                self.edit = Some(Edit {
                    field,
                    value: field.color(&mut self.choices[self.selected]).clone(),
                    replace: true,
                    invalid_paste: false,
                });
            }
            KeyCode::Enter => return Action::Save,
            KeyCode::Esc | KeyCode::Char('q') => return Action::Cancel,
            _ => (),
        }
        Action::Continue
    }

    fn paste(&mut self, text: &str) {
        if let Some(edit) = &mut self.edit {
            let text = text.trim();
            let value = if text.starts_with('#') {
                text.to_owned()
            } else {
                format!("#{text}")
            };
            edit.invalid_paste = rgb(&value).is_err();
            if !edit.invalid_paste {
                edit.value = value;
                edit.replace = false;
            }
        }
    }

    fn draw(&self, out: &mut impl Write, colors: bool) -> Result<()> {
        write!(out, "  MOOD SHELL  /  prompt colors and layout\r\n\r\n")?;
        for (index, mood) in self.choices.iter().enumerate() {
            write!(
                out,
                "  {} {}{}\r\n",
                if index == self.selected { ">" } else { " " },
                mood.name,
                if index == NAMES.len() {
                    " (saved)"
                } else if mood.name == "paper" {
                    " (light terminal)"
                } else {
                    ""
                }
            )?;
        }
        let mut mood = self.preview();
        let path_color = if self
            .edit
            .as_ref()
            .is_some_and(|edit| edit.field == Field::Path && rgb(&edit.value).is_ok())
        {
            PathColor::Theme
        } else {
            self.path.color
        };
        write!(
            out,
            "\r\n  Preview / {}\r\n  {}\r\n  {}\r\n\r\n",
            if mood.layout == Layout::Compact {
                "compact"
            } else {
                "two-line"
            },
            prompt::render(
                &mood,
                self.path.sample(),
                0,
                Shell::Plain,
                colors,
                path_color
            )
            .replace('\n', "\r\n  "),
            prompt::render(
                &mood,
                self.path.sample(),
                1,
                Shell::Plain,
                colors,
                path_color
            )
            .replace('\n', "\r\n  ")
        )?;
        for (index, field) in Field::ALL.iter().enumerate() {
            write!(
                out,
                "  {}  {}  {}\r\n",
                index + 1,
                if *field == Field::Path && path_color == PathColor::Terminal {
                    "terminal"
                } else {
                    field.color(&mut mood)
                },
                field.label()
            )?;
        }
        if let Some(edit) = &self.edit {
            write!(
                out,
                "\r\n  Edit {}: {}_\r\n  {}\r\n  Enter apply color   Esc discard color   Ctrl-C cancel all",
                edit.field.label(),
                edit.value,
                if edit.invalid_paste {
                    "Paste a single #RRGGBB color, for example #C4A7E7."
                } else if edit.replace {
                    "Type or paste #RRGGBB to replace. Backspace to edit."
                } else if rgb(&edit.value).is_ok() {
                    "Preview updated. Color is ready to apply."
                } else {
                    "Enter 6 hex digits (0-9, A-F). Ctrl-U clears."
                }
            )?;
        } else {
            write!(
                out,
                "\r\n  Up/down choose   Tab layout   1-4 color   P path: theme/terminal\r\n  Enter save   Esc cancel   (nothing saved until Enter)\r\n  {}",
                if colors {
                    "Styles the prompt; terminal background stays the same."
                } else {
                    "Color preview disabled by NO_COLOR or TERM=dumb."
                }
            )?;
        }
        Ok(())
    }
}

pub fn run(path: &Path) -> Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        bail!(
            "The picker needs an interactive terminal. Try `moodsh theme list` or `moodsh theme set dusk`"
        );
    }
    let config = config::load(path)?;
    let mut picker = Picker::new(config.mood);
    picker.path = config.path;
    let colors =
        std::env::var_os("NO_COLOR").is_none() && std::env::var("TERM").as_deref() != Ok("dumb");
    terminal::enable_raw_mode()?;
    let screen = Screen;
    execute!(
        io::stdout(),
        EnterAlternateScreen,
        cursor::Hide,
        EnableBracketedPaste
    )?;
    let save = loop {
        let mut out = io::stdout();
        execute!(out, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
        let (width, height) = terminal::size()?;
        let too_small = width < 80 || height < 24;
        if too_small {
            write!(out, "Resize to 80 x 24.\r\nEsc cancels.")?;
        } else {
            picker.draw(&mut out, colors)?;
        }
        out.flush()?;
        let event = event::read().context("Cannot read keyboard input")?;
        if let Event::Paste(text) = &event
            && !too_small
        {
            picker.paste(text);
        }
        if let Event::Key(key) = event {
            // Do not accept edits or saves while the controls are hidden.
            if too_small {
                if key.kind != KeyEventKind::Release
                    && (key.code == KeyCode::Esc
                        || (key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL)))
                {
                    break false;
                }
                continue;
            }
            match picker.key(key) {
                Action::Save => break true,
                Action::Cancel => break false,
                Action::Continue => (),
            }
        }
    };
    drop(screen);
    if save {
        let mood = picker.preview();
        // A config made invalid while the picker was open must not be overwritten.
        let mut config = config::load(path)?;
        config.mood = mood.clone();
        config.path.color = picker.path.color;
        config::save(path, &config)?;
        println!("Saved {}. Your next prompt will use it.", mood.name);
    } else {
        println!("Kept your current mood.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(picker: &mut Picker, code: KeyCode) -> Action {
        picker.key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    fn type_text(picker: &mut Picker, text: &str) {
        for c in text.chars() {
            key(picker, KeyCode::Char(c));
        }
    }

    #[test]
    fn terminal_path_preview_survives_browsing_and_cancelled_color_edit() {
        let mut picker = Picker::new(Mood::builtin("paper").unwrap());
        key(&mut picker, KeyCode::Char('p'));
        assert_eq!(picker.path.color, PathColor::Terminal);
        key(&mut picker, KeyCode::Up);
        let mut out = Vec::new();
        picker.draw(&mut out, true).unwrap();
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("\x1b[39m~/code/moodsh")
        );
        type_text(&mut picker, "2ABCDEF");
        key(&mut picker, KeyCode::Esc);
        assert_eq!(picker.path.color, PathColor::Terminal);
        type_text(&mut picker, "2ABCDEF");
        key(&mut picker, KeyCode::Enter);
        assert_eq!(picker.path.color, PathColor::Theme);
        assert_eq!(picker.preview().palette.path, "#ABCDEF");
    }

    #[test]
    fn colors_preview_before_apply_and_escape_discards() {
        let mut picker = Picker::new(Mood::default());
        let original = picker.preview();
        type_text(&mut picker, "1#12abEF");
        assert_eq!(picker.preview().palette.accent, "#12abEF");
        assert_eq!(picker.choices[0], original);
        key(&mut picker, KeyCode::Esc);
        assert_eq!(picker.preview(), original);
        type_text(&mut picker, "112abEF");
        assert_eq!(key(&mut picker, KeyCode::Enter), Action::Continue);
        assert_eq!(picker.choices[0].palette.accent, "#12abEF");
        assert_eq!(key(&mut picker, KeyCode::Enter), Action::Save);
    }

    #[test]
    fn incomplete_and_unsafe_input_never_reaches_renderer_or_save() {
        let mut picker = Picker::new(Mood::default());
        type_text(&mut picker, "2#12\u{1b}\n$%🙂");
        assert_eq!(picker.preview(), Mood::default());
        assert_eq!(key(&mut picker, KeyCode::Enter), Action::Continue);
        assert!(picker.edit.is_some());
        type_text(&mut picker, "3456");
        assert_eq!(picker.preview().palette.path, "#123456");
        type_text(&mut picker, "789");
        assert_eq!(picker.preview().palette.path, "#123456");
        key(&mut picker, KeyCode::Backspace);
        assert_eq!(picker.preview(), Mood::default());
        picker.key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
        type_text(&mut picker, "ABCDEF");
        assert_eq!(picker.preview().palette.path, "#ABCDEF");
    }

    #[test]
    fn edits_survive_browsing_and_preserve_other_saved_fields() {
        let mut current = Mood::builtin("ocean").unwrap();
        current.name = "afterhours".into();
        current.layout = Layout::TwoLine;
        current.palette.path = "#112233".into();
        let mut picker = Picker::new(current.clone());
        for (shortcut, field) in Field::ALL.into_iter().enumerate() {
            type_text(&mut picker, &format!("{}#AABBCC", shortcut + 1));
            key(&mut picker, KeyCode::Enter);
            *field.color(&mut current) = "#AABBCC".into();
            assert_eq!(picker.preview(), current);
        }
        key(&mut picker, KeyCode::Down);
        key(&mut picker, KeyCode::Up);
        assert_eq!(picker.preview(), current);
        type_text(&mut picker, "1FFFFFF");
        assert_eq!(
            picker.key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Action::Cancel
        );
    }

    #[test]
    fn release_events_do_not_double_apply_or_save_on_windows() {
        let mut picker = Picker::new(Mood::default());
        type_text(&mut picker, "1FFFFFF");
        key(&mut picker, KeyCode::Enter);
        let mut release = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        release.kind = KeyEventKind::Release;
        assert_eq!(picker.key(release), Action::Continue);
    }

    #[test]
    fn full_editor_fits_minimum_terminal() {
        let mood = Mood {
            name: "a".repeat(48),
            layout: Layout::TwoLine,
            ..Mood::default()
        };
        let mut picker = Picker::new(mood);
        for editing in [false, true] {
            if editing {
                type_text(&mut picker, "3");
            }
            let mut out = Vec::new();
            picker.draw(&mut out, false).unwrap();
            let text = String::from_utf8(out).unwrap();
            assert!(text.lines().count() <= 24, "{text}");
            assert!(text.lines().all(|line| line.len() < 80), "{text}");
        }
    }

    #[test]
    fn pasted_text_is_validated_and_cannot_trigger_save() {
        let mut picker = Picker::new(Mood::default());
        picker.paste("#ABCDEF\r\n");
        assert_eq!(picker.preview(), Mood::default());
        type_text(&mut picker, "1");
        picker.paste("#ABCDEF\r\n");
        assert_eq!(picker.preview().palette.accent, "#ABCDEF");
        assert!(picker.edit.is_some());
        picker.paste("\x1b[31m#123456\n\n");
        assert!(picker.edit.as_ref().unwrap().invalid_paste);
        assert_eq!(picker.preview().palette.accent, "#ABCDEF");
        assert_eq!(key(&mut picker, KeyCode::Enter), Action::Continue);
        assert!(picker.edit.is_some());
        picker.paste("123456");
        assert_eq!(picker.preview().palette.accent, "#123456");
    }
}
