use std::fmt::Debug;
use std::time::Duration;

use chrono::{self, DateTime, Datelike, TimeZone, Utc};
use reqwest::{self, blocking::Client};
use serde_json;
use thiserror::Error;

use crate::api::{self, chessdotcom, lichessdotorg, Api, Game, Games};

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("HTTP Error")]
    HTTPError(#[from] reqwest::Error),
    #[error("Reqwest client failed to build")]
    ClientBuildError(#[source] reqwest::Error),
    #[error("There was an error with the request chess API")]
    ApiError(#[from] api::ApiError),
    #[error("Failed to deserialize JSON response")]
    JSONDeserializationError(#[from] serde_json::Error),
}

pub struct ChessClient {
    client: Client,
    api: Api,
}

impl ChessClient {
    pub fn new(timeout: u64, api: &str) -> Result<Self, ClientError> {
        let timeout = Duration::new(timeout, 0);

        Ok(ChessClient {
            client: Client::builder()
                .timeout(timeout)
                .build()
                .map_err(|source| ClientError::ClientBuildError(source))?,
            api: Api::from_str(api).expect("Unsupported API"),
        })
    }

    pub fn get_user_month_games(
        &self,
        username: &str,
        year: i32,
        month: u32,
    ) -> Result<Games, ClientError> {
        log::info!("Requesting games for {} at {}/{}", username, month, year);
        let from = Utc.ymd(year, month, 1 as u32).and_hms(0, 0, 0);
        let to = first_day_next_month(from);

        let request = self.api.user_games(username, from, to)?;

        let response = self.client.execute(request)?;
        log::debug!("Response: {:?}", response);
        log::debug!(
            "Response length: {}",
            response.content_length().unwrap_or(0 as u64)
        );

        match self.api {
            Api::ChessDotCom => {
                let games = response.json::<chessdotcom::Games>()?;
                Ok(Games::ChessDotCom(games.games))
            }
            Api::LichessDotOrg => {
                let games = response
                    .text()?
                    .split("\n")
                    .map(|s| serde_json::from_str(s).unwrap())
                    .collect::<Vec<lichessdotorg::Game>>();
                Ok(Games::LichessDotOrg(games))
            }
        }
    }

    pub fn get_user_game_archives(
        &self,
        username: &str,
    ) -> Result<chessdotcom::GameArchives, ClientError> {
        log::info!("Requesting archives for {}", username);
        let request = self.api.user_archives(username)?;
        let response = self.client.execute(request)?;
        log::debug!("Response: {:?}", response);
        log::debug!(
            "Response length: {}",
            response.content_length().unwrap_or(0 as u64)
        );
        let archives: chessdotcom::GameArchives = response.json()?;
        log::debug!("Archives: {:?}", archives);
        Ok(archives)
    }

    pub fn get_last_user_game(&self, username: &str) -> Result<Game, ClientError> {
        log::info!("Requesting last game for {}", username);
        let request = self.api.last_user_game(username)?;

        let response = self.client.execute(request)?;
        log::debug!("Response: {:?}", response);
        log::debug!(
            "Response length: {}",
            response.content_length().unwrap_or(0 as u64)
        );
        let text = response.text()?;
        log::debug!("Response text: {}", text);
        let game: lichessdotorg::Game = serde_json::from_str(&text)?;
        Ok(Game::LichessDotOrg(game))
    }

    pub fn get_game(&self, id: &str) -> Result<Game, ClientError> {
        log::info!("Requesting game id {}", id);
        let request = self.api.game(id)?;
        let response = self.client.execute(request)?;
        log::debug!("Response: {:?}", response);
        log::debug!(
            "Response length: {}",
            response.content_length().unwrap_or(0 as u64)
        );
        let game = match self.api {
            Api::ChessDotCom => {
                Game::ChessDotComLive(response.json::<chessdotcom::CallbackLiveGame>()?)
            }
            Api::LichessDotOrg => Game::LichessDotOrg(response.json::<lichessdotorg::Game>()?),
        };
        Ok(game)
    }
}

fn first_day_next_month<D: Datelike>(d: D) -> DateTime<Utc> {
    if d.month() == 12 {
        Utc.ymd(d.year() + 1, 1, 1).and_hms(0, 0, 0)
    } else {
        Utc.ymd(d.year(), d.month() + 1, 1).and_hms(0, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_day_next_month() {
        let d = Utc.ymd(2020, 12, 1).and_hms(0, 0, 0);
        assert_eq!(
            first_day_next_month(d),
            Utc.ymd(2021, 1, 1).and_hms(0, 0, 0)
        );

        let d = Utc.ymd(2020, 10, 1);
        assert_eq!(
            first_day_next_month(d),
            Utc.ymd(2020, 11, 1).and_hms(0, 0, 0)
        );
    }
}
