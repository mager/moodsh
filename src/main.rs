mod config;
mod picker;
mod prompt;
mod shell;
mod theme;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use std::io::{self, IsTerminal, Write};
use theme::{Layout, Mood};

#[derive(Parser)]
#[command(
    version,
    about = "Mood Shell — make your shell feel like yours",
    after_help = "Get started: moodsh theme list\nTry a mood:  moodsh theme preview dusk\nSet it up:   moodsh init zsh"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print shell integration. Add it to your shell profile (see README).
    Init {
        #[arg(value_enum)]
        shell: prompt::Shell,
    },
    /// Browse, preview, and save moods.
    Theme {
        #[command(subcommand)]
        command: ThemeCommand,
    },
    /// Pick a mood and prompt layout with a live terminal preview.
    Customize,
    /// Render a prompt (used by shell integrations).
    Prompt {
        #[arg(long, value_enum, default_value = "plain")]
        shell: prompt::Shell,
        #[arg(long, default_value_t = 0, allow_hyphen_values = true)]
        status: i32,
        #[arg(long)]
        no_color: bool,
    },
    /// Show the config path or its effective contents.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Check your configuration and terminal environment.
    Doctor,
}

#[derive(Subcommand)]
enum ConfigCommand {
    Path,
    Show,
}

#[derive(Subcommand)]
enum ThemeCommand {
    /// List the built-in moods.
    List,
    /// Preview a built-in mood, or your current mood if omitted.
    Preview {
        name: Option<String>,
        #[arg(long, value_enum)]
        layout: Option<Layout>,
    },
    /// Save a built-in mood, optionally overriding its colors and layout.
    Set {
        name: String,
        #[command(flatten)]
        overrides: Overrides,
    },
}

#[derive(Args)]
struct Overrides {
    #[arg(long, value_name = "#RRGGBB")]
    accent: Option<String>,
    #[arg(long, value_name = "#RRGGBB")]
    path: Option<String>,
    #[arg(long, value_name = "#RRGGBB")]
    muted: Option<String>,
    #[arg(long, value_name = "#RRGGBB")]
    error: Option<String>,
    #[arg(long, value_enum)]
    layout: Option<Layout>,
}

fn colors_enabled() -> bool {
    std::env::var_os("NO_COLOR").is_none() && std::env::var("TERM").as_deref() != Ok("dumb")
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init { shell: target } => {
            print!("{}", shell::init(target, &std::env::current_exe()?)?)
        }
        Command::Theme {
            command: ThemeCommand::List,
        } => {
            println!("Mood Shell / built-in moods\n");
            for name in theme::NAMES {
                let mood = Mood::builtin(name)?;
                println!("  {name:8} {}", mood.palette.accent);
            }
            println!("\nPreview: moodsh theme preview dusk\nChoose:  moodsh customize");
        }
        Command::Theme {
            command: ThemeCommand::Preview { name, layout },
        } => {
            let mut mood = match name {
                Some(name) => Mood::builtin(&name)?,
                None => config::load(&config::path()?)?.mood,
            };
            if let Some(layout) = layout {
                mood.layout = layout;
            }
            let colors = colors_enabled() && io::stdout().is_terminal();
            println!("{}\n", mood.name);
            println!(
                "{}git status",
                prompt::render(&mood, "~/code/moodsh", 0, prompt::Shell::Plain, colors)
            );
            println!(
                "{}",
                prompt::render(&mood, "~/code/moodsh", 1, prompt::Shell::Plain, colors)
            );
            println!(
                "\naccent {}  path {}  muted {}  error {}",
                mood.palette.accent, mood.palette.path, mood.palette.muted, mood.palette.error
            );
        }
        Command::Theme {
            command: ThemeCommand::Set { name, overrides },
        } => {
            let mut mood = Mood::builtin(&name)?;
            if let Some(value) = overrides.accent {
                mood.palette.accent = value;
            }
            if let Some(value) = overrides.path {
                mood.palette.path = value;
            }
            if let Some(value) = overrides.muted {
                mood.palette.muted = value;
            }
            if let Some(value) = overrides.error {
                mood.palette.error = value;
            }
            if let Some(value) = overrides.layout {
                mood.layout = value;
            }
            let path = config::path()?;
            // Avoid silently destroying a malformed or newer configuration.
            config::load(&path)?;
            config::save(&path, &config::Config { mood })?;
            println!(
                "Saved {name} to {}. Your next prompt will use it.",
                path.display()
            );
        }
        Command::Customize => picker::run(&config::path()?)?,
        Command::Prompt {
            shell,
            status,
            no_color,
        } => {
            let config = config::load(&config::path()?)?;
            let cwd = std::env::current_dir().context("Cannot read current directory")?;
            let path = prompt::directory(&cwd, dirs::home_dir().as_deref());
            // Shells capture stdout: IsTerminal would incorrectly disable prompt colors.
            print!(
                "{}",
                prompt::render(
                    &config.mood,
                    &path,
                    status,
                    shell,
                    colors_enabled() && !no_color
                )
            );
        }
        Command::Config { command } => {
            let path = config::path()?;
            match command {
                ConfigCommand::Path => println!("{}", path.display()),
                ConfigCommand::Show => print!("{}", toml::to_string_pretty(&config::load(&path)?)?),
            }
        }
        Command::Doctor => {
            let path = config::path()?;
            let config = config::load(&path)?;
            println!(
                "Mood Shell {}\nPlatform: {} / {}",
                env!("CARGO_PKG_VERSION"),
                std::env::consts::OS,
                std::env::consts::ARCH
            );
            println!(
                "Config: {} ({})",
                path.display(),
                if path.exists() {
                    "valid"
                } else {
                    "using defaults"
                }
            );
            println!(
                "Mood: {}\nColors: {}",
                config.mood.name,
                if colors_enabled() {
                    "enabled (24-bit terminal recommended)"
                } else {
                    "disabled by NO_COLOR or TERM=dumb"
                }
            );
            println!("Binary: {}", std::env::current_exe()?.display());
            println!("Shell integration: add the init line from README to your shell profile.");
        }
    }
    io::stdout().flush()?;
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        if error
            .downcast_ref::<io::Error>()
            .is_some_and(|e| e.kind() == io::ErrorKind::BrokenPipe)
        {
            return;
        }
        eprintln!("moodsh: {error:#}");
        std::process::exit(1);
    }
}
