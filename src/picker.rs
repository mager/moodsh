use crate::{
    config::{self, Config},
    prompt::{self, Shell},
    theme::{Layout, Mood, NAMES},
};
use anyhow::{Context, Result, bail};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
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
        let _ = execute!(io::stdout(), cursor::Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

pub fn run(path: &Path) -> Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        bail!(
            "The picker needs an interactive terminal. Try `moodsh theme list` or `moodsh theme set dusk`"
        );
    }
    let config = config::load(path)?;
    let mut choices = NAMES
        .iter()
        .map(|name| Mood::builtin(name).expect("built-in mood"))
        .collect::<Vec<_>>();
    // Keep the exact saved palette available, including custom color overrides.
    let mut selected = choices
        .iter()
        .position(|mood| mood == &config.mood)
        .unwrap_or_else(|| {
            choices.push(config.mood.clone());
            choices.len() - 1
        });
    let mut layout = config.mood.layout;
    let colors =
        std::env::var_os("NO_COLOR").is_none() && std::env::var("TERM").as_deref() != Ok("dumb");
    terminal::enable_raw_mode()?;
    let screen = Screen;
    execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;
    let save = loop {
        let mut out = io::stdout();
        execute!(out, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
        write!(out, "\r\n  MOOD SHELL  /  make yourself at home\r\n\r\n")?;
        for (index, mood) in choices.iter().enumerate() {
            writeln!(
                out,
                "  {} {}\r",
                if index == selected { ">" } else { " " },
                mood.name
            )?;
        }
        let mut mood = choices[selected].clone();
        mood.layout = layout;
        write!(
            out,
            "\r\n  Preview\r\n\r\n  {}\r\n\r\n  {}\r\n",
            prompt::render(&mood, "~/code/moodsh", 0, Shell::Plain, colors).replace('\n', "\r\n  "),
            prompt::render(&mood, "~/code/moodsh", 1, Shell::Plain, colors).replace('\n', "\r\n  ")
        )?;
        write!(
            out,
            "\r\n  Up/down choose   Tab layout   Enter save   Esc cancel\r\n"
        )?;
        out.flush()?;
        if let Event::Key(key) = event::read().context("Cannot read keyboard input")? {
            if key.kind == KeyEventKind::Release {
                continue;
            }
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    selected = (selected + choices.len() - 1) % choices.len()
                }
                KeyCode::Down | KeyCode::Char('j') => selected = (selected + 1) % choices.len(),
                KeyCode::Tab => {
                    layout = if layout == Layout::Compact {
                        Layout::TwoLine
                    } else {
                        Layout::Compact
                    }
                }
                KeyCode::Enter => break true,
                KeyCode::Esc | KeyCode::Char('q') => break false,
                KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                    break false;
                }
                _ => (),
            }
        }
    };
    drop(screen);
    if save {
        let mut mood = choices.swap_remove(selected);
        mood.layout = layout;
        config::save(path, &Config { mood: mood.clone() })?;
        println!("Saved {}. Your next prompt will use it.", mood.name);
    } else {
        println!("Kept your current mood.");
    }
    Ok(())
}
