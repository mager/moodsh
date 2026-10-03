mod config;
mod picker;
mod prompt;
mod shell;
mod shortcuts;
mod theme;
mod theme_file;

use anyhow::{Context, Result};
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::Shell as CompletionShell;
use std::{
    io::{self, IsTerminal, Write},
    path::PathBuf,
};
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
    #[command(name = "__git-main-branch", hide = true)]
    GitMainBranch,
    /// Print shell integration. Add it to your shell profile (see README).
    Init {
        #[arg(value_enum)]
        shell: InitShell,
        /// Add opt-in shortcuts; existing aliases, functions, and commands win.
        #[arg(long, value_enum)]
        shortcuts: Option<ShortcutSet>,
    },
    /// Show the exact commands in a shortcut set.
    Shortcuts {
        #[arg(value_enum)]
        set: ShortcutSet,
    },
    /// Browse, preview, and save moods.
    Theme {
        #[command(subcommand)]
        command: ThemeCommand,
    },
    /// Pick a mood, edit its colors, and choose a layout with a live preview.
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
    /// Preview a built-in mood, a Markdown theme file, or your current mood.
    Preview {
        #[arg(value_parser = theme::NAMES)]
        name: Option<String>,
        #[arg(long, value_name = "PATH", conflicts_with = "name")]
        file: Option<PathBuf>,
        #[arg(long, value_enum)]
        layout: Option<Layout>,
    },
    /// Save a built-in mood, optionally overriding its colors and layout.
    Set {
        #[arg(value_parser = theme::NAMES)]
        name: String,
        #[command(flatten)]
        overrides: Overrides,
    },
    /// Create a Markdown theme from a built-in mood. Never overwrites a file.
    New {
        file: PathBuf,
        #[arg(long, default_value = "dusk", value_parser = theme::NAMES)]
        from: String,
        /// Theme name; defaults to the filename without its extension.
        #[arg(long)]
        name: Option<String>,
    },
    /// Export your current palette and layout as a shareable Markdown theme.
    Export {
        file: PathBuf,
        /// Optionally give the exported theme a new name.
        #[arg(long)]
        name: Option<String>,
    },
    /// Validate and apply a local Markdown theme to your prompt.
    Apply { file: PathBuf },
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

#[derive(Clone, Copy, ValueEnum)]
enum ShortcutSet {
    Git,
}

#[derive(Clone, Copy, ValueEnum)]
enum InitShell {
    Zsh,
    Bash,
    Powershell,
}

impl From<InitShell> for prompt::Shell {
    fn from(shell: InitShell) -> Self {
        match shell {
            InitShell::Zsh => Self::Zsh,
            InitShell::Bash => Self::Bash,
            InitShell::Powershell => Self::Powershell,
        }
    }
}

fn colors_enabled() -> bool {
    std::env::var_os("NO_COLOR").is_none() && std::env::var("TERM").as_deref() != Ok("dumb")
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::GitMainBranch => println!("{}", shortcuts::git_main_branch()?),
        Command::Init {
            shell: target,
            shortcuts: enabled,
        } => {
            let script = shell::init(target.into(), &std::env::current_exe()?)?;
            let completion_shell = match target {
                InitShell::Zsh => CompletionShell::Zsh,
                InitShell::Bash => CompletionShell::Bash,
                InitShell::Powershell => CompletionShell::PowerShell,
            };
            let shortcut_script = enabled
                .map(|_| shortcuts::init(target.into(), &std::env::current_exe()?))
                .transpose()?;
            let mut completions = Vec::new();
            clap_complete::generate(
                completion_shell,
                &mut Cli::command(),
                "moodsh",
                &mut completions,
            );
            let mut stdout = io::stdout().lock();
            // PowerShell requires `using namespace` declarations at the top of the script.
            if matches!(target, InitShell::Powershell) {
                stdout.write_all(&completions)?;
                stdout.write_all(script.as_bytes())?;
            } else {
                stdout.write_all(script.as_bytes())?;
                stdout.write_all(&completions)?;
            }
            if let Some(shortcuts) = shortcut_script {
                stdout.write_all(shortcuts.as_bytes())?;
            }
        }
        Command::Shortcuts {
            set: ShortcutSet::Git,
        } => {
            println!("moodsh / Git shortcuts (opt-in)\n");
            for (name, command) in shortcuts::GIT {
                println!("  {name:6} {command}");
            }
            println!("  gprom  git pull --rebase origin <main-branch> (resolved at invocation)");
            println!(
                "\nEnable: add --shortcuts git to your moodsh init line, then open a new shell.\nExisting command names win. PowerShell usually keeps its built-in gc, gl, and gp.\nDisable: remove the flag and open a new shell. Git must be installed separately."
            );
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
            command: ThemeCommand::Preview { name, file, layout },
        } => {
            let mut mood = match (name, file) {
                (_, Some(path)) => theme_file::load(&path)?,
                (Some(name), None) => Mood::builtin(&name)?,
                (None, None) => config::load(&config::path()?)?.mood,
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
        Command::Theme {
            command: ThemeCommand::New { file, from, name },
        } => {
            let mut mood = Mood::builtin(&from)?;
            mood.name = match name {
                Some(name) => name,
                None => file
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .context("Use --name to give this theme an ASCII name")?
                    .to_owned(),
            };
            theme_file::write(&file, &mood)?;
            println!(
                "Created {}. Edit its moodsh block, then preview with `moodsh theme preview --file PATH`.",
                file.display()
            );
        }
        Command::Theme {
            command: ThemeCommand::Export { file, name },
        } => {
            let mut mood = config::load(&config::path()?)?.mood;
            if let Some(name) = name {
                mood.name = name;
            }
            theme_file::write(&file, &mood)?;
            println!(
                "Exported {} to {}. Your active prompt is unchanged.",
                mood.name,
                file.display()
            );
        }
        Command::Theme {
            command: ThemeCommand::Apply { file },
        } => {
            let mood = theme_file::load(&file)?;
            let path = config::path()?;
            config::load(&path)?;
            let name = mood.name.clone();
            config::save(&path, &config::Config { mood })?;
            println!(
                "Applied {name}. Your next prompt will use it. The Markdown file is unchanged."
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
        // Theme files and paths can contain untrusted terminal control characters.
        let diagnostic: String = format!("{error:#}")
            .chars()
            .flat_map(|c| {
                if (c.is_control() && c != '\n' && c != '\t')
                    || matches!(c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
                {
                    c.escape_default().collect::<Vec<_>>()
                } else {
                    vec![c]
                }
            })
            .collect();
        eprintln!("moodsh: {diagnostic}");
        std::process::exit(1);
    }
}
