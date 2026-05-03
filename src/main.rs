mod commands;
pub mod store;

use anyhow::Result;
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use time::macros::format_description;
use tracing::metadata::LevelFilter;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::registry;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(
    author,
    version = option_env!("VERGEN_GIT_DESCRIBE").unwrap_or(env!("CARGO_PKG_VERSION")),
    about,
    long_about = None,
    after_help = concat!(
        "\n",
        "✨ Rustin is ready to turn chaos into checkmarks ✨\n",
        "🦀 Feed me tasks. I crave productivity.\n",
        "📋 Tiny board, big energy.\n",
        "🚀 Ship it, finish it, then pretend it was easy."
    ),
    help_template = concat!(
        "{name} {version}\n",
        "{about}\n\n",
        "Author: {author}\n",
        "License: ",
        env!("CARGO_PKG_LICENSE"),
        "\n\n",
        "{usage-heading} {usage}\n\n",
        "{all-args}{after-help}"
    )
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task
    #[command(visible_alias = "a")]
    Add(commands::add::AddCommand),
    /// Remove an existing task
    #[command(visible_alias = "r")]
    Remove(commands::remove::RemoveCommand),
    /// List all tasks
    #[command(visible_alias = "l")]
    List(commands::list::ListCommand),
    /// Move a task to the 'todo' column
    #[command(visible_alias = "t")]
    Todo(commands::todo::TodoCommand),
    /// Move a task to the 'in-progress' column
    #[command(visible_alias = "ip")]
    Inprogress(commands::inprogress::InprogressCommand),
    /// Move a task to the 'done' column
    #[command(visible_alias = "d")]
    Done(commands::done::DoneCommand),
    /// Show all fields of a single task
    #[command(visible_alias = "s")]
    Show(commands::show::ShowCommand),
    /// Show board activity statistics
    #[command(visible_alias = "st")]
    Stat(commands::stat::StatCommand),
    /// Edit fields of a task (except history)
    #[command(visible_alias = "e")]
    Edit(commands::edit::EditCommand),
    /// Initialize a new board or set its title
    Init(commands::init::InitCommand),
    /// Open an interactive terminal UI
    #[command(visible_alias = "ui")]
    Tui(commands::tui::TuiCommand),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let timer = UtcTime::new(format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second]"
    ));

    let level = cli.verbose.tracing_level_filter();

    if level >= LevelFilter::DEBUG {
        let indicatif_layer = IndicatifLayer::new();
        registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .compact()
                    .with_writer(indicatif_layer.get_stdout_writer())
                    .with_timer(timer),
            )
            .with(indicatif_layer)
            .with(level)
            .init();
    } else {
        let indicatif_layer = IndicatifLayer::new();
        registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .compact()
                    .with_writer(indicatif_layer.get_stdout_writer())
                    .without_time()
                    .with_level(false)
                    .with_target(false),
            )
            .with(indicatif_layer)
            .with(level)
            .init();
    }

    let command = cli
        .command
        .unwrap_or(Commands::List(commands::list::ListCommand {
            columns: vec![],
        }));

    match command {
        Commands::Add(cmd) => cmd.run()?,
        Commands::Remove(cmd) => cmd.run()?,
        Commands::List(cmd) => cmd.run()?,
        Commands::Todo(cmd) => cmd.run()?,
        Commands::Inprogress(cmd) => cmd.run()?,
        Commands::Done(cmd) => cmd.run()?,
        Commands::Show(cmd) => cmd.run()?,
        Commands::Stat(cmd) => cmd.run()?,
        Commands::Edit(cmd) => cmd.run()?,
        Commands::Init(cmd) => cmd.run()?,
        Commands::Tui(cmd) => cmd.run()?,
    }

    Ok(())
}
