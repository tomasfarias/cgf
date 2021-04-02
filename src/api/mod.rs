use std::fmt::Debug;

use chrono::{self, DateTime, Datelike, Utc};
use reqwest::{self, blocking::Request, Method, Url};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json;
use thiserror::Error;

pub mod chessdotcom;
pub mod lichessdotorg;

pub trait ChessPlayer {
    fn name(&self) -> String;
    fn title(&self) -> Option<String>;
    fn rating(&self) -> u32;
    fn url(&self) -> Option<String>;
    fn result(&self) -> Option<String>;
}

/// Trait encompassing minimum information expected from all APIs: a PGN, a white
/// player, a black player, a URL, and the time when the game was played.
pub trait ChessGame {
    type PlayerType: ChessPlayer;

    fn to_json_pretty(&self) -> Result<String, serde_json::Error>;
    fn to_json(&self) -> Result<String, serde_json::Error>;
    fn pgn(&mut self) -> String;
    fn white(&mut self) -> Self::PlayerType;
    fn black(&mut self) -> Self::PlayerType;
    fn url(&self) -> String;
    fn end_time(&self) -> DateTime<Utc>;
}

/// A supertrait encompassing required traits for proper displaying of a chess
/// game, in either JSON, PGN, or table format.
pub trait DisplayableChessGame: ChessGame + Serialize + DeserializeOwned + Clone + Debug {}

#[derive(Debug, Clone, Deserialize)]
pub enum Games {
    ChessDotCom(Vec<chessdotcom::Game>),
    LichessDotOrg(Vec<lichessdotorg::Game>),
}

#[derive(Debug, Clone, Deserialize)]
pub enum Player {
    ChessDotCom(chessdotcom::Player),
    ChessDotComLive(chessdotcom::LivePlayer),
    LichessDotOrg(lichessdotorg::Player),
}

impl ChessPlayer for Player {
    fn name(&self) -> String {
        match self {
            Player::ChessDotCom(p) => p.name(),
            Player::ChessDotComLive(p) => p.name(),
            Player::LichessDotOrg(p) => p.name(),
        }
    }

    fn title(&self) -> Option<String> {
        match self {
            Player::ChessDotCom(p) => p.title(),
            Player::ChessDotComLive(p) => p.title(),
            Player::LichessDotOrg(p) => p.title(),
        }
    }

    fn rating(&self) -> u32 {
        match self {
            Player::ChessDotCom(p) => p.rating(),
            Player::ChessDotComLive(p) => p.rating(),
            Player::LichessDotOrg(p) => p.rating(),
        }
    }

    fn url(&self) -> Option<String> {
        match self {
            Player::ChessDotCom(p) => p.url(),
            Player::ChessDotComLive(p) => p.url(),
            Player::LichessDotOrg(p) => p.url(),
        }
    }

    fn result(&self) -> Option<String> {
        match self {
            Player::ChessDotCom(p) => p.result(),
            Player::ChessDotComLive(p) => p.result(),
            Player::LichessDotOrg(p) => p.result(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Game {
    ChessDotCom(chessdotcom::Game),
    ChessDotComLive(chessdotcom::CallbackLiveGame),
    LichessDotOrg(lichessdotorg::Game),
}

impl ChessGame for Game {
    type PlayerType = Player;

    fn to_json(&self) -> Result<String, serde_json::Error> {
        match self {
            Game::ChessDotCom(g) => g.to_json(),
            Game::ChessDotComLive(g) => g.to_json(),
            Game::LichessDotOrg(g) => g.to_json(),
        }
    }

    fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        match self {
            Game::ChessDotCom(g) => g.to_json_pretty(),
            Game::ChessDotComLive(g) => g.to_json_pretty(),
            Game::LichessDotOrg(g) => g.to_json_pretty(),
        }
    }

    fn pgn(&mut self) -> String {
        match self {
            Game::ChessDotCom(g) => g.pgn(),
            Game::ChessDotComLive(g) => g.pgn(),
            Game::LichessDotOrg(g) => g.pgn(),
        }
    }

    fn white(&mut self) -> Self::PlayerType {
        match self {
            Game::ChessDotCom(g) => Player::ChessDotCom(g.white()),
            Game::ChessDotComLive(g) => Player::ChessDotComLive(g.white()),
            Game::LichessDotOrg(g) => Player::LichessDotOrg(g.white()),
        }
    }

    fn black(&mut self) -> Self::PlayerType {
        match self {
            Game::ChessDotCom(g) => Player::ChessDotCom(g.black()),
            Game::ChessDotComLive(g) => Player::ChessDotComLive(g.black()),
            Game::LichessDotOrg(g) => Player::LichessDotOrg(g.black()),
        }
    }

    fn url(&self) -> String {
        match self {
            Game::ChessDotCom(g) => g.url(),
            Game::ChessDotComLive(g) => g.url(),
            Game::LichessDotOrg(g) => g.url(),
        }
    }

    fn end_time(&self) -> DateTime<Utc> {
        match self {
            Game::ChessDotCom(g) => g.end_time(),
            Game::ChessDotComLive(g) => g.end_time(),
            Game::LichessDotOrg(g) => g.end_time(),
        }
    }
}

impl DisplayableChessGame for Game {}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("{api:?} is not supported")]
    UnsupportedApi { api: String },
    #[error("endpoint {endpoint:?} not implemented for {api:?}")]
    EndpointNotImplemented { endpoint: String, api: String },
    #[error("URL could not be parsed")]
    URLParseFailed(#[from] url::ParseError),
    #[error("HTTP Error")]
    HTTPError(#[from] reqwest::Error),
}

#[derive(PartialEq, Debug)]
pub enum Api {
    ChessDotCom,
    LichessDotOrg,
}

impl Api {
    pub fn from_str(s: &str) -> Result<Self, ApiError> {
        match s {
            "chess.com" => Ok(Api::ChessDotCom),
            "lichess.org" => Ok(Api::LichessDotOrg),
            api => Err(ApiError::UnsupportedApi {
                api: api.to_string(),
            }),
        }
    }

    pub fn game(&self, id: &str) -> Result<Request, ApiError> {
        let url = match self {
            Api::ChessDotCom => {
                Url::parse(&format!("https://www.chess.com/callback/live/game/{}", id))?
            }
            Api::LichessDotOrg => Url::parse(&format!("https://lichess.org/game/export/{}", id))?,
        };
        Ok(Request::new(Method::GET, url))
    }

    pub fn user_archives(&self, username: &str) -> Result<Request, ApiError> {
        match self {
            Api::ChessDotCom => {
                let url = Url::parse(&format!(
                    "https://api.chess.com/pub/player/{}/games/archives",
                    username
                ))?;
                Ok(Request::new(Method::GET, url))
            }
            Api::LichessDotOrg => Err(ApiError::EndpointNotImplemented {
                endpoint: "/{user}/games/archives".to_string(),
                api: "lichess".to_string(),
            }),
        }
    }

    pub fn user_games(
        &self,
        username: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Request, ApiError> {
        match self {
            Api::ChessDotCom => {
                let month = from.month();
                let year = from.year();
                let month_str = month_string(month);
                let url = Url::parse(&format!(
                    "https://api.chess.com/pub/player/{}/games/{}/{}",
                    username,
                    year.to_string(),
                    month_str
                ))?;

                Ok(Request::new(Method::GET, url))
            }
            Api::LichessDotOrg => {
                let params = [
                    ("evals", "true"),
                    ("pgnInJson", "true"),
                    ("clocks", "true"),
                    ("opening", "true"),
                    ("since", &from.timestamp().to_string()),
                    ("until", &to.timestamp().to_string()),
                ];
                let url = Url::parse_with_params(
                    &format!("https://lichess.org/api/games/user/{}", username),
                    &params,
                )?;
                Ok(Request::new(Method::GET, url))
            }
        }
    }
}

/// Convert a month number into a 2 character string.
fn month_string(m: u32) -> String {
    if m < 10 {
        let mut zero: String = "0".to_owned();
        zero.push_str(&m.to_string());
        zero
    } else {
        m.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_month_string() {
        assert_eq!(month_string(10), "10".to_string());
        assert_eq!(month_string(2), "02".to_string());
        assert_eq!(month_string(9), "09".to_string());
    }

    #[test]
    fn test_chess_dot_com_api_game_endpoint_request() {
        let api = Api::from_str("chess.com").expect("should not break");
        // Parsing URL should not break
        let expected = Url::parse("https://www.chess.com/callback/live/game/101").unwrap();
        let result = api.game("101").unwrap();
        assert_eq!(result.url(), &expected);
        assert_eq!(result.method(), &Method::GET);
    }

    #[test]
    fn test_lichess_dot_org_api_game_endpoint_request() {
        let api = Api::from_str("lichess.org").expect("should not break");
        // Parsing URL should not break
        let expected = Url::parse("https://lichess.org/game/export/101").unwrap();
        let result = api.game("101").unwrap();
        assert_eq!(result.url(), &expected);
        assert_eq!(result.method(), &Method::GET);
    }

    #[test]
    fn test_chess_dot_com_api_user_archives_endpoint_request() {
        let api = Api::from_str("chess.com").expect("should not break");
        // Parsing URL should not break
        let expected = Url::parse("https://api.chess.com/pub/player/user1/games/archives").unwrap();
        let result = api.user_archives("user1").unwrap();
        assert_eq!(result.url(), &expected);
        assert_eq!(result.method(), &Method::GET);
    }

    #[test]
    fn test_chess_dot_com_api_user_games_endpoint_request() {
        let api = Api::from_str("chess.com").expect("should not break");
        let from = Utc.ymd(2020, 9, 1).and_hms(0, 0, 0);
        let to = Utc.ymd(2020, 10, 1).and_hms(0, 0, 0);
        // Parsing URL should not break
        let expected = Url::parse("https://api.chess.com/pub/player/user1/games/2020/09").unwrap();
        let result = api.user_games("user1", from, to).unwrap();
        assert_eq!(result.url(), &expected);
        assert_eq!(result.method(), &Method::GET);
    }

    #[test]
    fn test_lichess_dot_org_api_user_games_endpoint_request() {
        let api = Api::from_str("lichess.org").expect("should not break");
        let from = Utc.ymd(2020, 9, 1).and_hms(0, 0, 0);
        let to = Utc.ymd(2020, 10, 1).and_hms(0, 0, 0);
        // Parsing URL should not break
        let expected = Url::parse("https://lichess.org/api/games/user/user1?evals=true&pgnInJson=true&clocks=true&opening=true&since=1598918400&until=1601510400").unwrap();
        let result = api.user_games("user1", from, to).unwrap();
        assert_eq!(result.url(), &expected);
        assert_eq!(result.method(), &Method::GET);
    }

    #[test]
    #[should_panic]
    fn test_unsupported_api() {
        // Assuming there will never be an "unsupported" Api variant
        Api::from_str("unsupported").unwrap();
    }
}
