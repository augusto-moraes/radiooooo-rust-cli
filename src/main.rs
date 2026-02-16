// cargo run -- --decades 1960,1980 --moods SLOW,FAST --countries FRA,ITA --player mpv
// cargo run

/*
Installation: 
    cargo build --release && cp "./target/release/radiooooo-rust-cli" "/usr/local/bin/radiooooo"
Usage:
    radiooooo [-r] [--mode MODE] ... 
    (you might have to install mpv player if you don't have it yet)
    run : brew install mpv  (macOS)
    run : sudo apt install mpv (Debian/Ubuntu)

    consider adding the line `NEXT quit` to your mpv config file (~/.config/mpv/mpv.conf) 
    to allow skiping songs with your headphone remotelly
*/

use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use tokio::signal;
use clap::{Parser};
use colored::*;
use inquire::{Select, MultiSelect};
use std::{process::{Command, Stdio}, vec};

mod cli;
use crate::cli::Cli;

mod island_manager;
use crate::island_manager::{select_island, Island};

const SONG_URL: &str = "https://radiooooo.com/play";

#[derive(Debug, Deserialize)]
struct ApiResponse {
    error: Option<String>,
    links: Option<Links>,
    mood: Option<String>,
    title: Option<String>,
    artist: Option<String>,
    country: Option<String>,
    year: Option<String>,
    profile_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Links {
    mpeg: Option<String>,
}

async fn get_profile(client: &Client, profile_id: &str) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("https://radiooooo.com/contributor/{}", profile_id);

    let response = client.get(&url).send().await?;
    let json_resp: serde_json::Value = response.json().await?;

    let profile_name = json_resp["pseudonym"].as_str().unwrap_or("Unknown Profile").to_string();

    Ok(profile_name)
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

    let moods = if mode != "random" {
        MultiSelect::new(
            "Select mood: (space selects, enter to confirm)",
            vec!["SLOW", "FAST", "WEIRD"],
        )
        .prompt() 
        .unwrap()
    } else {
        vec![]
    };

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
                    "Exploring 🧭🌎 songs within".green(),
                    decades.is_empty()
                        .then(|| "ALL DECADES".yellow().to_string())
                        .unwrap_or_else(|| decades.join(", ").yellow().to_string()),
                    
                    moods.join(", ").yellow(),
                    countries.is_empty()
                        .then(|| "EVERYWHERE".yellow().to_string())
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
                        "ISLAND 🏝️🌞 with moods".green(), moods.join(", ").yellow());

                json!({
                    "mode": mode,
                    "island": island.id,
                    "moods": moods
                })
            },
            "taxi" => {
                println!(
                    "{} {} {} {} {} - {} - {}",
                    "Playing in".green(),
                    "TAXI MODE".yellow().bold(),
                    "🚖🌎",
                    "with".green(),
                    decades.is_empty()
                        .then(|| "ALL DECADES".yellow().to_string())
                        .unwrap_or_else(|| decades.join(", ").yellow().to_string()),
                    
                    moods.join(", ").yellow(),
                    countries.is_empty()
                        .then(|| "EVERYWHERE".yellow().to_string())
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
                    "{} {} {}",
                    "Playing in".green(),
                    "SHUFFLE MODE".cyan().bold(),
                    "🚀✨"
                );
                json!({
                    "mode": mode,
                    "moods": moods // TODO : it looks like shuffle mode only requires moods but ignores it
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
            "Now playing".green(),
            json_resp.title.unwrap_or_else(|| "Unknown".to_string()).blue().italic(),
            "by".green(),
            json_resp.artist.unwrap_or_else(|| "Unknown".to_string()).blue().italic(),
            json_resp.country.unwrap_or_else(|| "Unknown".to_string()).yellow(),
            json_resp.year.unwrap_or_else(|| "Unknown".to_string()).yellow(),
            json_resp.mood.unwrap_or_else(|| "Unknown".to_string()).yellow(),
        );
        println!(
            "{} {} [{}]",
            "Curated by ".green(),
            get_profile(&client, json_resp.profile_id.as_deref().unwrap_or_default()).await?
                .cyan().italic(),
            json_resp.profile_id.unwrap_or_default().yellow()
        );
        println!(
            "{} {}",
            "Link:".green(),
            song_url.bright_black().underline()
        );

        print!("{}{}{}", "Press ".magenta(), "Ctrl+C".magenta().bold(), " to exit ".magenta());
        println!("{}{}{}", "or ".magenta(), "q".magenta().bold(), " to skip to the next song".magenta());
        println!();

        println!("{} {}", "Player:".bright_black(), player.bright_black().bold());

        let status = Command::new(player)
            .arg("--no-video")
            .arg(&song_url)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .status()
            .expect("[Error] Failed to start mpv. Do you have mpv installed?");

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

    if cli.explore || cli.random || cli.decades.is_some() || cli.moods.is_some() || cli.countries.is_some() {
        run_direct(cli).await;
    } else {
        run_interactive(cli).await; 
    }
}
