use std::fmt;
use std::time::Duration;

use chrono::serde::ts_seconds::deserialize as from_ts;
use chrono::serde::ts_seconds_option::deserialize as from_ts_option;
use chrono::{self, DateTime, Utc};
use prettytable::Table;
use reqwest::{self};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub username: String,
    pub rating: u16,
    pub result: String,
    #[serde(rename(serialize = "id", deserialize = "@id"))]
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Game {
    pub white: Player,
    pub black: Player,
    pub url: String,
    pub fen: String,
    pub pgn: String,
    #[serde(deserialize_with = "from_ts_option")]
    #[serde(default)]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "from_ts")]
    pub end_time: DateTime<Utc>,
    pub time_control: String,
    pub rules: String,
    pub eco: Option<String>,
    pub tournament: Option<String>,
    pub r#match: Option<String>,
}

impl fmt::Display for Game {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        let mut game_table = Table::new();
        game_table.add_row(row![
            format!("{} ({}) ♔", self.white.username, self.white.rating),
            "vs.",
            format!("{} ({}) ♚", self.black.username, self.black.rating),
        ]);

        game_table.add_row(row![
            format!("{}", self.white.result),
            "",
            format!("{}", self.black.result),
        ]);

        game_table.printstd();
        Ok(())
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Games {
    pub games: Vec<Game>,
}

#[derive(Deserialize, Debug)]
pub struct GameArchives {
    pub archives: Vec<String>,
}

pub struct ChessApiClient {
    client: reqwest::blocking::Client,
}

impl ChessApiClient {
    pub fn new(timeout: u64) -> Result<Self, reqwest::Error> {
        let timeout = Duration::new(timeout, 0);

        Ok(ChessApiClient {
            client: reqwest::blocking::Client::builder()
                .timeout(timeout)
                .build()?,
        })
    }

    pub fn get_month_games(
        &self,
        username: &str,
        year: u32,
        month: u32,
    ) -> Result<Vec<Game>, reqwest::Error> {
        log::info!("Requesting games for {} at {}/{}", username, month, year);
        let month_string = if month < 10 {
            let mut zero: String = "0".to_owned();
            zero.push_str(&month.to_string());
            zero
        } else {
            month.to_string()
        };

        let request_url = format!(
            "https://api.chess.com/pub/player/{}/games/{}/{}",
            username,
            &year.to_string(),
            &month_string,
        );

        let response = self.client.get(&request_url).send()?;
        log::debug!("Response: {:?}", response);
        log::debug!(
            "Response length: {}",
            response.content_length().unwrap_or(0 as u64)
        );
        let games: Games = response.json()?;
        log::debug!("Games: {:?}", games);
        Ok(games.games)
    }

    pub fn get_archives(&self, username: &str) -> Result<GameArchives, reqwest::Error> {
        log::info!("Requesting archives for {}", username);
        let request_url = format!(
            "https://api.chess.com/pub/player/{}/games/archives",
            username
        );

        let response = self.client.get(&request_url).send()?;
        log::debug!("Response: {:?}", response);
        log::debug!(
            "Response length: {}",
            response.content_length().unwrap_or(0 as u64)
        );
        let archives: GameArchives = response.json()?;
        log::debug!("Archives: {:?}", archives);
        Ok(archives)
    }
}
