// cargo run -- --decades 1960,1980 --moods SLOW,FAST --countries FRA,ITA --player mpv
// cargo run

/*
cargo build --release
sudo cp "./target/release/radiooooo-rust-cli" "/usr/local/bin/radiooooo"
*/

use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use tokio::signal;

const SONG_URL: &str = "https://radiooooo.com/play";
// const FORMAT: &str = "mpeg";
const MODE: &str = "explore";

use clap::{Parser, ArgAction};
use colored::*;
use log::LevelFilter;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// Decades (comma separated)
    #[arg(long)]
    pub decades: Option<String>,

    /// Moods (comma separated)
    #[arg(long)]
    pub moods: Option<String>,

    /// Countries (comma separated ISO codes)
    #[arg(long)]
    pub countries: Option<String>,

    /// Audio player
    #[arg(long, default_value = "mpv")]
    pub player: String,

    #[arg(
        long,
        default_value_t = false,
        help = "Play songs in random order (default: false)"
    )]
    pub random: bool,

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

// fn default_player() -> &'static str {
//     // if cfg!(target_os = "macos") {
//     //     "play"
//     // } else {
//     //     "mpv"
//     // }
//     "mpv"
// }

#[derive(Debug, Deserialize)]
struct ApiResponse {
    error: Option<String>,
    links: Option<Links>,
}

#[derive(Debug, Deserialize)]
struct Links {
    mpeg: Option<String>,
}

use inquire::MultiSelect;

async fn run_interactive(cli: Cli) {
    println!("{}", "[Info] Letting selections empty to selects options".cyan());
    let decades = MultiSelect::new(
        "Select decade(s): (space selects, enter to confirm)",
        vec!["1900", "1910", "1920", "1930", "1940", "1950", "1960", 
                      "1970", "1980", "1990", "2000", "2010", "2020", "2070"],
    )
    .prompt()
    .unwrap();

    let moods = MultiSelect::new(
        "Select mood: (space selects, enter to confirm)",
        vec!["SLOW", "FAST", "WEIRD"],
    )
    .prompt() 
    .unwrap();

    let countries = MultiSelect::new(
        "Select country: (space selects, enter to confirm)",
        vec!["FRA", "USA", "ITA", "JPN", "BRA", "GBR", 
            "DEU", "ESP", "RUS", "CAN", "MEX", "IND" , 
            "CHN", "AUS" , "ARG" , "KOR", "SWE", "NLD", 
            "BEL", "CHE", "AUT", "NOR", "DNK", "FIN",
            "IRL", "PRT", "GRC", "TUR", "POL", "CZE",
            "HUN", "ROU", "BGR", "SRB", "UKR", "BLR",
            "EGY", "ZAF", "NGA", "KEN", "MAR", "TUN",
            "SAU", "ISR", "ARE", "IRN", "PAK", "BGD",
            "THA", "VNM", "IDN", "PHL", "NZL", "SGP", 
            "HKG", "TWN", "COL", "PER", "CHL", "VEN",
            "MYS", "LKA", "NPL", "LBN", "JOR", "KWT"
        ],
    )
    .prompt()
    .unwrap();

    let moods: Vec<&str> = moods.is_empty()
        .then(|| vec!["SLOW", "FAST", "WEIRD"])
        .unwrap_or_else(|| moods.iter().map(|s| *s).collect());

    let _ = play_loop(
        &cli.player,
        decades,
        moods,
        countries,
    )
    .await;
}

async fn run_direct(cli: Cli) {

    if cli.random {
        println!("{}", "[Info] Random mode selected, all options will be used".cyan());
        println!("{}", "       Bon Voyage !!".cyan().bold());

        let _ = play_loop(
            &cli.player,
            vec![],
            vec!["SLOW", "FAST", "WEIRD"],
            vec![],
        )
        .await;
        return;
    }

    let decades = cli.decades.expect("Missing --decades");
    let moods = cli.moods.expect("Missing --moods");
    let countries = cli.countries.expect("Missing --countries");

    let decades: Vec<&str> = decades.split(',').collect();
    let moods: Vec<&str> = moods.split(',').collect();
    let countries: Vec<&str> = countries.split(',').collect();

    let _ = play_loop(&cli.player, decades, moods, countries).await;
}


use std::process::{Command, Stdio};

async fn play_loop (
    player: &str,
    decades: Vec<&str>,
    moods: Vec<&str>,
    countries: Vec<&str>,
)   -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    loop {
        log::info!("Fetching song…");

        println!();
        println!(
            "{} {} - {} - {}",
            "Feching a new song for".green(),
            decades.is_empty()
                .then(|| "ALL DECADES".yellow().to_string())
                .unwrap_or_else(|| decades.join(", ").yellow().to_string()),
            
            moods.join(", ").yellow(),
            countries.is_empty()
                .then(|| "ALL COUNTRIES".yellow().to_string())
                .unwrap_or_else(|| countries.join(", ").yellow().to_string()),
        );

        let payload = json!({
            "mode": MODE,
            "moods": moods,
            "decades": decades,
            "isocodes": countries
        });

        let response = tokio::select! {
            res = client.post(SONG_URL).json(&payload).send() => res?,
            _ = signal::ctrl_c() => {
                println!("\nExited!");
                break Ok(());
            }
        };

        let json_resp: ApiResponse = response.json().await?;

        if let Some(err) = json_resp.error {
            eprintln!("Error: {}", err);
            break Err(err.into());
        }

        let song_url = match json_resp.links.and_then(|l| l.mpeg) {
            Some(url) => url,
            None => {
                eprintln!("No audio link found");
                continue;
            }
        };

        println!(
            "{} {}",
            "Now playing:".green(),
            song_url.blue().italic()
        );

        print!("{}{}{}", "Press ".magenta(), "Ctrl+C".magenta().bold(), " to exit ".magenta());
        println!("{}{}{}", "or ".magenta(), "q".magenta().bold(), " to skip to the next song".magenta());
        println!();

        let status = Command::new(player)
            .arg("--no-video")
            .arg(&song_url)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .status()
            .expect("Failed to start mpv");

        // print!("\n{:?}\n", status);

        if !status.success() {
            println!();
            if status.code() == Some(4) {
                println!("{}", "See you ;)".green());
                break Ok(());
            }
            log::error!("mpv exited with {}", status);
            break Ok(());
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.log_level())
        .init();

    println!(
        "{} {}",
        "radiooooo-cli".bold().green(),
        env!("CARGO_PKG_VERSION").bright_black()
    );

    println!(
        "{}{}{}{}{}{}{}{}\n{}",
        "Welcome to ",
        "radi".blue().bold(), "o".bold().red(), "o".bold().blue(),
        "o".bold().green(), "o".bold().magenta(), "o".bold().cyan(),
        "-cli".blue().bold(),
        "A command-line client made on RUST for radiooooo.com".bright_black()
    );

    if !cli.random && (cli.decades.is_none() || cli.moods.is_none() || cli.countries.is_none()) {
        run_interactive(cli).await;
    } else {
        run_direct(cli).await;
    }
}
