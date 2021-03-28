use std::fmt::Debug;
use std::time::Duration;

use chrono::serde::ts_seconds::deserialize as from_ts;
use chrono::serde::ts_seconds_option::deserialize as from_ts_option;
use chrono::{self, DateTime, Datelike, TimeZone, Utc};
use reqwest::{self, blocking::Client, blocking::Request, Method, Url};
use serde::{Deserialize, Serialize};
use shakmaty::{fen::Fen, CastlingMode, Chess, Color, Setup};
use thiserror::Error;
use url;

use crate::utils::next_move;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Player {
    pub username: String,
    pub rating: u16,
    pub result: String,
    #[serde(alias = "@id")]
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LivePlayer {
    pub username: String,
    pub rating: u16,
    pub id: u64,
    pub is_content_hidden: bool,
    pub avatar_url: String,
    pub country_id: i32,
    pub is_enabled: bool,
    pub can_win_on_time: bool,
    pub chess_title: Option<String>,
    pub color: String,
    pub country_name: String,
    pub default_tab: i32,
    pub has_moved_at_least_once: bool,
    pub is_drawable: bool,
    pub is_online: bool,
    pub is_in_live_chess: Option<bool>,
    pub is_touch_move: bool,
    pub is_vacation: bool,
    pub is_white_on_bottom: bool,
    #[serde(deserialize_with = "from_ts_option")]
    #[serde(default)]
    pub last_login_date: Option<DateTime<Utc>>,
    pub location: Option<String>,
    pub membership_level: Option<i32>,
    pub membership_code: Option<String>,
    #[serde(deserialize_with = "from_ts_option")]
    #[serde(default)]
    pub member_since: Option<DateTime<Utc>>,
    pub post_move_action: String,
    pub turn_time_remaining: String,
    pub flair_code: String,
    pub vacation_remaining: String,
    pub games_in_progress: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LivePlayers {
    pub top: LivePlayer,
    pub bottom: LivePlayer,
}

pub trait ChessGame {
    fn pgn(&mut self) -> String;
    fn white(&mut self) -> Player;
    fn black(&mut self) -> Player;
    fn url(&self) -> String;
    fn time(&self) -> DateTime<Utc>;
}

pub trait DisplayableChessGame: ChessGame + Serialize + Clone + Debug {}

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

impl ChessGame for Game {
    fn pgn(&mut self) -> String {
        self.pgn.clone()
    }

    fn white(&mut self) -> Player {
        self.white.clone()
    }

    fn black(&mut self) -> Player {
        self.black.clone()
    }

    fn url(&self) -> String {
        self.url.clone()
    }

    fn time(&self) -> DateTime<Utc> {
        self.end_time.clone()
    }
}

impl DisplayableChessGame for Game {}

#[derive(Deserialize, Debug, Clone)]
pub struct Games {
    pub games: Vec<Game>,
}

#[derive(Deserialize, Debug)]
pub struct GameArchives {
    pub archives: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Arena {
    name: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct PGNHeaders {
    pub event: String,
    pub site: String,
    pub date: String,
    pub white: String,
    pub black: String,
    pub result: String,
    #[serde(rename(deserialize = "ECO"))]
    pub eco: String,
    pub white_elo: i32,
    pub black_elo: i32,
    pub time_control: String,
    pub end_time: String,
    pub termination: String,
    pub set_up: String,
    #[serde(rename(deserialize = "FEN"))]
    pub fen: String,
    pub variant: Option<String>,
}

impl PGNHeaders {
    pub fn to_pgn_string(&self) -> String {
        let mut headers = String::new();
        headers.push_str(&format!("[Event {}]\n", self.event));
        headers.push_str(&format!("[Site {}]\n", self.site));
        headers.push_str(&format!("[Date {}]\n", self.date));
        headers.push_str(&format!("[White {}]\n", self.white));
        headers.push_str(&format!("[Black {}]\n", self.black));
        headers.push_str(&format!("[Result {}]\n", self.result));
        headers.push_str(&format!("[CurrentPosition {}]\n", self.fen));
        headers.push_str(&format!("[ECO {}]\n", self.eco));
        headers.push_str(&format!("[WhiteElo {}]\n", self.white_elo));
        headers.push_str(&format!("[BlackElo {}]\n", self.black_elo));
        headers.push_str(&format!("[TimeControl {}]\n", self.time_control));
        headers.push_str(&format!("[EndTime {}]\n", self.end_time));
        headers.push_str(&format!("[Termination {}]\n\n", self.termination));
        headers
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LiveGame {
    pub can_send_trophy: bool,
    pub changes_players_rating: i32,
    pub color_of_winner: Option<String>,
    pub id: u64,
    pub initial_setup: String,
    pub is_live_game: bool,
    pub is_abortable: bool,
    pub is_analyzable: bool,
    pub is_checkmate: bool,
    pub is_stalemate: bool,
    pub is_finished: bool,
    pub is_rated: bool,
    pub is_resignable: bool,
    pub last_move: String,
    pub move_list: String,
    pub ply_count: i32,
    pub rating_change_white: i32,
    pub rating_change_black: i32,
    pub result_message: String,
    #[serde(deserialize_with = "from_ts")]
    pub end_time: DateTime<Utc>,
    pub arena: Option<Arena>,
    pub turn_color: String,
    pub r#type: String,
    pub type_name: String,
    pub allow_vacation: bool,
    pub pgn_headers: PGNHeaders,
    pub move_timestamps: String,
    pub base_time_1: i32,
    pub time_increment_1: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CallbackLiveGame {
    pub players: LivePlayers,
    pub game: LiveGame,
}

impl DisplayableChessGame for CallbackLiveGame {}

impl CallbackLiveGame {
    pub fn get_result_code(&self, color: &str) -> String {
        let base_player = if self.players.top.color.as_str() == color {
            &self.players.top
        } else {
            &self.players.bottom
        };

        if let Some(c) = &self.game.color_of_winner {
            // Somebody won and somebody lost
            if c == color {
                "win".to_string()
            } else if self.game.is_checkmate {
                "checkmated".to_string()
            } else if base_player.turn_time_remaining == "Out of time" {
                "timeout".to_string()
            } else if self.game.result_message.contains("resignation") {
                "resigned".to_string()
            } else {
                "lose".to_string()
            }
        } else {
            // Draw happened for many reasons
            if self.game.is_stalemate {
                "stalemate".to_string()
            } else if self.game.result_message == "Game drawn by repetition" {
                "repetition".to_string()
            } else if self.game.result_message == "Game drawn by insufficient material" {
                "insufficient".to_string()
            } else if self.game.result_message == "Game drawn by agreement" {
                "agreed".to_string()
            } else {
                // Missing some variation rules
                "timevsinsufficient".to_string()
            }
        }
    }
}

impl ChessGame for CallbackLiveGame {
    fn pgn(&mut self) -> String {
        let setup: Fen = self.game.pgn_headers.fen.parse().unwrap();
        let mut position: Chess = setup.position(CastlingMode::Standard).unwrap();

        let mut counter = 1;
        let mut pgn = String::new();
        let mut moves: Vec<char> = self.game.move_list.chars().rev().collect();

        pgn.push_str(&self.game.pgn_headers.to_pgn_string());
        loop {
            let m = next_move(&mut moves, &mut position);

            if m.is_none() {
                break;
            }

            if position.turn() == Color::Black {
                pgn.push_str(&counter.to_string());
                pgn.push('.');
                counter += 1;
            }
            pgn.push_str(&m.unwrap());
            pgn.push(' ');
        }

        pgn.push_str(&self.game.pgn_headers.result);

        String::from(pgn)
    }

    fn white(&mut self) -> Player {
        match self.players.top.color.as_str() {
            "white" => Player {
                username: self.players.top.username.clone(),
                rating: self.players.top.rating,
                result: self.get_result_code(&"white"),
                id: format!(
                    "https://www.chess.com/member/{}",
                    self.players.top.username.clone()
                ),
            },
            _ => Player {
                username: self.players.bottom.username.clone(),
                rating: self.players.bottom.rating,
                result: self.get_result_code(&"white"),
                id: format!(
                    "https://www.chess.com/member/{}",
                    self.players.bottom.username.clone()
                ),
            },
        }
    }

    fn black(&mut self) -> Player {
        match self.players.top.color.as_str() {
            "black" => Player {
                username: self.players.top.username.clone(),
                rating: self.players.top.rating,
                result: self.get_result_code(&"black"),
                id: format!(
                    "https://www.chess.com/member/{}",
                    self.players.top.username.clone()
                ),
            },
            _ => Player {
                username: self.players.bottom.username.clone(),
                rating: self.players.bottom.rating,
                result: self.get_result_code(&"black"),
                id: format!(
                    "https://www.chess.com/member/{}",
                    self.players.bottom.username.clone()
                ),
            },
        }
    }

    fn url(&self) -> String {
        format!("https://www.chess.com/live/game/{}", self.game.id)
    }

    fn time(&self) -> DateTime<Utc> {
        self.game.end_time.clone()
    }
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("endpoint {endpoint:?} not implemented for {api:?}")]
    EndpointNotImplemented { endpoint: String, api: String },
    #[error("URL could not be parsed")]
    URLParseFailed(#[from] url::ParseError),
    #[error("HTTP Error")]
    HTTPError(#[from] reqwest::Error),
    #[error("Reqwest client failed to build")]
    ClientBuildError(#[source] reqwest::Error),
    #[error("{api:?} is not supported")]
    UnsupportedApi { api: String },
}

#[derive(PartialEq, Debug)]
pub enum Api {
    ChessDotCom,
    LichessDotOrg,
}

impl Api {
    pub fn from_str(s: &str) -> Result<Self, ClientError> {
        match s {
            "chess.com" => Ok(Api::ChessDotCom),
            "lichess.org" => Ok(Api::LichessDotOrg),
            api => Err(ClientError::UnsupportedApi {
                api: api.to_string(),
            }),
        }
    }

    pub fn game(&self, id: &str) -> Result<Request, ClientError> {
        let url = match self {
            Api::ChessDotCom => {
                Url::parse(&format!("https://www.chess.com/callback/live/game/{}", id))?
            }
            Api::LichessDotOrg => Url::parse(&format!("https://lichess.org/game/export/{}", id))?,
        };
        Ok(Request::new(Method::GET, url))
    }

    pub fn user_archives(&self, username: &str) -> Result<Request, ClientError> {
        match self {
            Api::ChessDotCom => {
                let url = Url::parse(&format!(
                    "https://api.chess.com/pub/player/{}/games/archives",
                    username
                ))?;
                Ok(Request::new(Method::GET, url))
            }
            Api::LichessDotOrg => Err(ClientError::EndpointNotImplemented {
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
    ) -> Result<Request, ClientError> {
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

pub struct ChessClient {
    client: Client,
    api: Api,
}

impl ChessClient {
    pub fn new(timeout: u64) -> Result<Self, ClientError> {
        let timeout = Duration::new(timeout, 0);

        Ok(ChessClient {
            client: Client::builder()
                .timeout(timeout)
                .build()
                .map_err(|source| ClientError::ClientBuildError(source))?,
            api: Api::ChessDotCom,
        })
    }

    pub fn get_user_month_games(
        &self,
        username: &str,
        year: i32,
        month: u32,
    ) -> Result<Vec<impl DisplayableChessGame>, ClientError> {
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
        let games: Games = response.json()?;
        log::debug!("Games: {:?}", games);
        Ok(games.games)
    }

    pub fn get_user_game_archives(&self, username: &str) -> Result<GameArchives, ClientError> {
        log::info!("Requesting archives for {}", username);
        let request = self.api.user_archives(username)?;
        let response = self.client.execute(request)?;
        log::debug!("Response: {:?}", response);
        log::debug!(
            "Response length: {}",
            response.content_length().unwrap_or(0 as u64)
        );
        let archives: GameArchives = response.json()?;
        log::debug!("Archives: {:?}", archives);
        Ok(archives)
    }

    pub fn get_game(&self, id: &str) -> Result<impl DisplayableChessGame, ClientError> {
        log::info!("Requesting game id {}", id);
        let request = self.api.game(id)?;
        let response = self.client.execute(request)?;
        log::debug!("Response: {:?}", response);
        log::debug!(
            "Response length: {}",
            response.content_length().unwrap_or(0 as u64)
        );
        let callback: CallbackLiveGame = response.json()?;
        log::debug!("Callback: {:?}", callback);
        Ok(callback)
    }
}

fn month_string(m: u32) -> String {
    if m < 10 {
        let mut zero: String = "0".to_owned();
        zero.push_str(&m.to_string());
        zero
    } else {
        m.to_string()
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
    fn test_month_string() {
        assert_eq!(month_string(10), "10".to_string());
        assert_eq!(month_string(2), "02".to_string());
        assert_eq!(month_string(9), "09".to_string());
    }

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
