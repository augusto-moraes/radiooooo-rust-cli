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
const ISLAND_MAP_URL: &str = "https://app.radiooooo.com/island/map";

use clap::{Parser, ArgAction};
use colored::*;
use log::LevelFilter;

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

#[derive(Debug, Deserialize)]
struct ApiResponse {
    error: Option<String>,
    links: Option<Links>,
    mood: Option<String>,
    title: Option<String>,
    artist: Option<String>,
    country: Option<String>,
    year: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Links {
    mpeg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Modified {
    date: String,
}

#[derive(Debug, Deserialize)]
struct Island {
    #[serde(rename = "_id")]
    id: String,

    name: String,

    category: Option<String>,

    modified: Option<Modified>,
}

use chrono::{DateTime, Utc};

fn island_modified_timestamp(island: &Island) -> DateTime<Utc> {
    island
        .modified
        .as_ref()
        .and_then(|m| DateTime::parse_from_rfc3339(&m.date).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or(DateTime::<Utc>::MIN_UTC)
}

fn sort_islands_by_modified(mut islands: Vec<Island>) -> Vec<Island> {
    islands.sort_by(|a, b| {
        island_modified_timestamp(b).cmp(&island_modified_timestamp(a))
    });
    islands
}

async fn fetch_islands() -> Result<Vec<Island>, reqwest::Error> {
    let client = Client::new();

    client
        .get(ISLAND_MAP_URL)
        .send()
        .await?
        .json::<Vec<Island>>()
        .await
}

fn island_labels(islands: &[Island]) -> Vec<String> {
    islands
        .iter()
        .map(|i| {
            let category = i.category.as_deref().unwrap_or("other");
            format!("{} [{}]", i.name, category)
        })
        .collect()
}

use inquire::{MultiSelect, Select};

async fn select_island() -> Island {
    let islands = fetch_islands()
        .await
        .expect("Failed to fetch islands");

    let islands = sort_islands_by_modified(islands);
    let labels = island_labels(&islands);

    let selected_label = Select::new(
        "Select an island:",
        labels,
    )
    .prompt()
    .expect("Selection aborted");

    // Map label → Island
    islands
        .into_iter()
        .find(|i| selected_label.starts_with(&i.name))
        .expect("Selected island not found")
}

async fn run_interactive(cli: Cli) {
    println!("{}", "[Info] Letting selections empty selects all options".cyan());
    let mode = Select::new(
        "Select mode:",
        vec!["explore", "islands", "taxi", "random"],
    )
    .prompt()
    .unwrap();

    let island = if mode == "islands" {
        Some(select_island().await)
    } else {
        None
    };

    let moods = MultiSelect::new(
        "Select mood: (space selects, enter to confirm)",
        vec!["SLOW", "FAST", "WEIRD"],
    )
    .prompt() 
    .unwrap();

    let decades = if mode == "explore" || mode == "taxi" {
        MultiSelect::new(
            "Select decade(s): (space selects, enter to confirm)",
            vec!["1900", "1910", "1920", "1930", "1940", "1950", "1960", 
                          "1970", "1980", "1990", "2000", "2010", "2020", "2070"],
        )
        .prompt()
        .unwrap()
    } else {
        vec![]
    };

    let countries = if mode == "explore" || mode == "taxi" {
        MultiSelect::new(
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
        .unwrap()
    } else {
        vec![]
    };

    let moods: Vec<&str> = moods.is_empty()
        .then(|| vec!["SLOW", "FAST", "WEIRD"])
        .unwrap_or_else(|| moods.iter().map(|s| *s).collect());

    let _ = play_loop(
        &cli.player,
        mode,
        decades,
        moods,
        countries,
        island
    )
    .await;
}

async fn run_direct(cli: Cli) {

    if cli.random {
        println!();
        println!("{}", "[Info] Random mode selected, all options will be used".cyan());
        println!("{}", "       Bon Voyage !!".cyan().bold());

        let _ = play_loop(
            &cli.player,
            "random", // TODO: study witch algorithm is better : /play/random or /play with everything selected
            vec![],
            vec!["SLOW", "FAST", "WEIRD"],
            vec![],
            None
        )
        .await;
        return;
    }

    let decades: Vec<&str> = cli.decades.as_ref()
        .map(|s| s.split(',').collect())
        .unwrap_or_default();
    
    let moods: Vec<String> = cli.moods.as_ref()
        .map(|s| s.split(',').map(|m| m.to_uppercase()).collect())
        .unwrap_or_else(|| vec!["SLOW".to_string(), "FAST".to_string(), "WEIRD".to_string()]);
    
    let countries: Vec<String> = cli.countries.as_ref()
        .map(|s| s.split(',').map(|c| c.to_uppercase()).collect())
        .unwrap_or_default();
    
    let moods: Vec<&str> = moods.iter().map(|s| s.as_str()).collect();
    let countries: Vec<&str> = countries.iter().map(|s| s.as_str()).collect();

    let _ = play_loop(&cli.player, &cli.mode, decades, moods, countries, None).await;
}


use std::{process::{Command, Stdio}, vec};

async fn play_loop (
    player: &str,
    mode: &str,
    decades: Vec<&str>,
    moods: Vec<&str>,
    countries: Vec<&str>,
    island: Option<Island>,
)   -> Result<(), Box<dyn std::error::Error>> {
    let client: Client = Client::new();

    loop {
        log::info!("Fetching song…");

        println!();
        
        let payload = match mode {
            "explore" => {
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
                json!({
                    "mode": mode,
                    "moods": moods,
                    "decades": decades,
                    "isocodes": countries
                })
            },
            "islands" => {
                let island = island.as_ref().expect("Island must be provided in islands mode");
                println!("{} {}[{}] {} {}", "Feching a new song from the".green(), 
                        island.name.cyan().bold(), island.category.as_deref().unwrap_or("other").cyan() ,
                        "ISLAND with moods".green(), moods.join(", ").yellow());

                json!({
                    "mode": mode,
                    "island": island.id,
                    "moods": moods
                })
            },
            "taxi" => {
                println!(
                    "{} {} {} {} - {} - {}",
                    "Playing in".green(),
                    "TAXI MODE".yellow().bold(),
                    "with".green(),
                    decades.is_empty()
                        .then(|| "ALL DECADES".yellow().to_string())
                        .unwrap_or_else(|| decades.join(", ").yellow().to_string()),
                    
                    moods.join(", ").yellow(),
                    countries.is_empty()
                        .then(|| "ALL COUNTRIES".yellow().to_string())
                        .unwrap_or_else(|| countries.join(", ").yellow().to_string()),
                );
                json!({
                    "mode": mode,
                    "moods": moods,
                    "decades": decades,
                    "isocodes": countries
                })
            },
            _ => {
                println!(
                    "{} {} {} {}",
                    "Playing in".green(),
                    "SHUFFLE MODE".cyan().bold(),
                    "with moods".green(),
                    moods.join(", ").yellow()
                );
                json!({
                    "mode": mode,
                    "moods": moods
                })
            }
        };

        let url = if mode == "random" {SONG_URL.to_owned() + "/random"} else {SONG_URL.to_string()};
        let response = tokio::select! {
            res = client.post(url).json(&payload).send() => res?,
            _ = signal::ctrl_c() => {
                println!("\nExited!");
                break Ok(());
            }
        };     

        // let raw_json = response.text().await?;
        // println!("Raw API response: {}", raw_json);

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
            "{} {} {} {} [{} - {} - {}]",
            "Now playing:".green(),
            json_resp.title.unwrap_or_else(|| "Unknown".to_string()).blue().italic(),
            "by".green(),
            json_resp.artist.unwrap_or_else(|| "Unknown".to_string()).blue().italic(),
            json_resp.country.unwrap_or_else(|| "Unknown".to_string()).yellow(),
            json_resp.year.unwrap_or_else(|| "Unknown".to_string()).yellow(),
            json_resp.mood.unwrap_or_else(|| "Unknown".to_string()).yellow(),
        );
        println!(
            "{} {}",
            "Link:".green(),
            song_url.bright_black().underline()
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

    println!();

    println!(
        "{}{}{}{}{}{}{}{}\n{}",
        "Welcome to ",
        "radi".blue().bold(), "o".bold().red(), "o".bold().blue(),
        "o".bold().green(), "o".bold().magenta(), "o".bold().cyan(),
        "-cli".blue().bold(),
        "A command-line client made on RUST for radiooooo.com".bright_black()
    );

    if cli.random || cli.decades.is_some() || cli.moods.is_some() || cli.countries.is_some() {
        run_direct(cli).await;
    } else {
       run_interactive(cli).await; 
    }
}
