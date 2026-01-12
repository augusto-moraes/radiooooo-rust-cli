// TODO : THIS WILL ONLY WORK IF THE USER IS LOGGED IN

const PROFILE_ID: &str = "65aad4a7b041ab7470eaa5c1";

const PLAYLISTS_URL: &str = "https://app.radiooooo.com/playlist/list?profile_id=";
const PLAYLIST_TRACKS_URL: &str = "https://app.radiooooo.com/playlist/track/"; // need playlist _id passed in json body
const TRACK_INFO_URL: &str = "https://app.radiooooo.com/track/play/"; // + track id

const LIKE: &str = "https://app.radiooooo.com/like/";
const LIKED_SONGS: &str = "https://app.radiooooo.com/contributor/likes/"; // + profile id

use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PlaylistInfo {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct RawPlaylist {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    #[serde(rename = "tracks")]
    pub track_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Playlist {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Deserialize)]
pub struct Track {
    #[serde(rename = "_id")]
    pub id: String,
    pub title: String,
    pub artist: String,
    pub country: String,
    pub year: String,
    pub mood: String,
    pub links: Option<Links>,
}

#[derive(Debug, Deserialize)]
pub struct Links {
    pub mpeg: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LikedTrack {
    #[serde(rename = "_id")]
    pub id: String,
    pub title: String,
    pub artist: String,
    pub country: String,
    pub year: String,
    pub mood: String,
}

pub async fn fetch_playlists() -> Result<Vec<PlaylistInfo>, reqwest::Error> {
    let client = Client::new();
    let url = format!("{}{}", PLAYLISTS_URL, PROFILE_ID);

    client
        .get(&url)
        .send()
        .await?
        .json::<Vec<PlaylistInfo>>()
        .await
}

pub async fn fetch_track(track_id: &str) -> Result<Track, reqwest::Error> {
    let client = Client::new();
    let url = format!("{}{}", TRACK_INFO_URL, track_id);

    client
        .get(&url)
        .send()
        .await?
        .json::<Track>()
        .await
}

pub async fn fetch_playlist_tracks(playlist_id: &str) -> Result<Playlist, reqwest::Error> {
    let client = Client::new();
    let url = PLAYLIST_TRACKS_URL;

    let body = serde_json::json!({
        "_id": playlist_id
    });

    let raw_result = client
        .post(url)
        .json(&body)
        .send()
        .await?
        .text()
        .await;

    println!("{raw_result:?}");

    // let mut tracks = Vec::new();
    // for track_id in raw_result.track_ids {
    //     let track = fetch_track(&track_id).await?;
    //     tracks.push(track);
    // }

    // Ok(Playlist{
    //     id: raw_result.id,
    //     name: raw_result.name,
    //     tracks,
    // })

    Ok(Playlist{
        id: "".to_string(),
        name: "raw_result.name".to_string(),
        tracks: vec![],
    })
}

// This wont work if user is not logged in
pub async fn fetch_liked_songs() -> Result<Vec<LikedTrack>, reqwest::Error> {
    let client = Client::new();
    let url = format!("{}{}", LIKED_SONGS, PROFILE_ID);

    client
        .get(&url)
        .send()
        .await?
        .json::<Vec<LikedTrack>>()
        .await
}

pub fn get_song_url(track: &Track) -> Option<String> {
    track.links.as_ref()?.mpeg.clone()
}

// pub fn select_playlist(playlists: &[PlaylistInfo]) -> Option<PlaylistInfo> {
//     let selections: Vec<String> = playlists
//         .iter()
//         .map(|p| format!("{} ({})", p.name, p.id))
//         .collect();

//     let selection = dialoguer::Select::new()
//         .with_prompt("Select a playlist")
//         .items(&selections)
//         .default(0)
//         .interact()
//         .ok()?;

//     playlists.get(selection).cloned()
// }

// pub fn select_track(playlist: &Playlist) -> Option<Track> {
//     let selections: Vec<String> = playlist
//         .tracks
//         .iter()
//         .map(|t| format!("{} - {} ({})", t.artist, t.title, t.id))
//         .collect();

//     let selection = dialoguer::Select::new()
//         .with_prompt("Select a track")
//         .items(&selections)
//         .default(0)
//         .interact()
//         .ok()?;

//     playlist.tracks.get(selection).cloned()
// }

pub fn play_track(player: &str, track_url: &str) -> Result<(), std::io::Error> {
    std::process::Command::new(player)
        .arg(track_url)
        .status()?;
    Ok(())
}

// pub fn loop_playlist_play_track(player: &str) -> Result<(), std::io::Error> {
//     loop {
//         let playlists = fetch_playlists()
//             .await
//             .expect("Failed to fetch playlists");

//         let selected_playlist = select_playlist(&playlists)
//             .expect("No playlist selected");

//         let playlist_tracks = fetch_playlist_tracks(&selected_playlist.id)
//             .await
//             .expect("Failed to fetch playlist tracks");

//         let selected_track = select_track(&playlist_tracks)
//             .expect("No track selected");

//         let track_url = get_song_url(&selected_track)
//             .expect("No track URL found");

//         std::process::Command::new(player)
//             .arg(track_url)
//             .status()?;
//     }

//     Ok (())
// }
