use log::LevelFilter;
use clap::{Parser, ArgAction};

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    // MODE
    #[arg(long, default_value = "explore")]
    pub mode: String,

    /// Decades (comma separated)
    #[arg(long, short = 'd')]
    pub decades: Option<String>,

    /// Moods (comma separated)
    #[arg(long, short = 'm')]
    pub moods: Option<String>,

    /// Countries (comma separated ISO codes)
    #[arg(long, short = 'c')]
    pub countries: Option<String>,

    /// Audio player
    #[arg(long, default_value = "mpv")]
    pub player: String,

    // --random or -r
    #[arg(
        long,
        default_value_t = false,
        help = "Play songs in random order (default: false)",
        action = ArgAction::SetTrue,
        short = 'r',
    )]
    pub random: bool,

    /// Explore mode
    #[arg(
        long,
        default_value_t = false,
        help = "Explore mode (default: false)",
        action = ArgAction::SetTrue,
        short = 'e',
    )]
    pub explore: bool,

    /// Verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = ArgAction::Count)]
    pub verbose: u8,
}

impl Cli {
    pub fn log_level(&self) -> LevelFilter {
        match self.verbose {
            0 => LevelFilter::Warn,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    }
}
