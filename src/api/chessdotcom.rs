use std::fmt::Debug;

use chrono::serde::ts_seconds::deserialize as from_ts;
use chrono::serde::ts_seconds_option::deserialize as from_ts_option;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use shakmaty::{fen::Fen, CastlingMode, Chess, Color, Setup};

use super::{ChessGame, ChessPlayer, DisplayableChessGame};

use crate::utils::next_move;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Player {
    pub username: String,
    pub rating: u32,
    pub result: String,
    #[serde(alias = "@id")]
    pub id: String,
}

impl ChessPlayer for Player {
    fn name(&self) -> String {
        self.username.clone()
    }

    fn title(&self) -> Option<String> {
        None
    }

    fn rating(&self) -> Option<u32> {
        Some(self.rating.clone())
    }

    fn url(&self) -> Option<String> {
        Some(self.id.clone())
    }

    fn result(&self) -> Option<String> {
        Some(self.result.clone())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LivePlayer {
    pub username: String,
    pub rating: u32,
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

impl ChessPlayer for LivePlayer {
    fn name(&self) -> String {
        self.username.clone()
    }

    fn title(&self) -> Option<String> {
        match &self.chess_title {
            Some(t) => Some(t.clone()),
            None => None,
        }
    }

    fn rating(&self) -> Option<u32> {
        Some(self.rating.clone())
    }

    fn url(&self) -> Option<String> {
        Some(format!(
            "https://www.chess.com/member/{}",
            self.username.clone()
        ))
    }

    fn result(&self) -> Option<String> {
        None
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LivePlayers {
    pub top: LivePlayer,
    pub bottom: LivePlayer,
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

impl ChessGame for Game {
    type PlayerType = Player;

    fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    fn pgn(&mut self) -> String {
        self.pgn.clone()
    }

    fn white(&mut self) -> Self::PlayerType {
        self.white.clone()
    }

    fn black(&mut self) -> Self::PlayerType {
        self.black.clone()
    }

    fn url(&self) -> String {
        self.url.clone()
    }

    fn end_time(&self) -> DateTime<Utc> {
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
    pub fn to_pgn_string(&self, id: &str) -> String {
        let mut headers = String::new();
        headers.push_str(&format!("[Event \"{}\"]\n", self.event));
        headers.push_str(&format!("[Site \"{}\"]\n", self.site));
        headers.push_str(&format!("[Date \"{}\"]\n", self.date));
        headers.push_str(&format!("[White \"{}\"]\n", self.white));
        headers.push_str(&format!("[Black \"{}\"]\n", self.black));
        headers.push_str(&format!("[Result \"{}\"]\n", self.result));
        headers.push_str(&format!("[CurrentPosition \"{}\"]\n", self.fen));
        headers.push_str(&format!("[ECO \"{}\"]\n", self.eco));
        headers.push_str(&format!("[WhiteElo \"{}\"]\n", self.white_elo));
        headers.push_str(&format!("[BlackElo \"{}\"]\n", self.black_elo));
        headers.push_str(&format!("[TimeControl \"{}\"]\n", self.time_control));
        headers.push_str(&format!("[EndTime \"{}\"]\n", self.end_time));
        headers.push_str(&format!("[Termination \"{}\"]\n", self.termination));
        headers.push_str(&format!(
            "[Link \"https://www.chess.com/game/live/{}\"]\n\n",
            id
        ));
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
    pub rating_change_white: Option<i32>,
    pub rating_change_black: Option<i32>,
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
    type PlayerType = LivePlayer;

    fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    fn pgn(&mut self) -> String {
        let setup: Fen = self.game.pgn_headers.fen.parse().unwrap();
        let mut position: Chess = setup.position(CastlingMode::Standard).unwrap();

        let mut counter = 1;
        let mut pgn = String::new();
        // This next loop should probably be handled by some iter implemenation
        let mut moves: Vec<char> = self.game.move_list.chars().rev().collect();
        let mut timestamps: Vec<u32> = self
            .game
            .move_timestamps
            .split(",")
            .map(|s| s.parse::<u32>().unwrap())
            .collect();
        timestamps.reverse();

        pgn.push_str(
            &self
                .game
                .pgn_headers
                .to_pgn_string(&self.game.id.to_string()),
        );
        loop {
            let m = next_move(&mut moves, &mut position);
            if m.is_none() {
                break;
            }

            let ts = timestamps.pop().unwrap();
            let (hours, minutes, secs, tenth_secs) = time_from_timestamp(ts);
            let clock_comment = format!(
                " {{[%clk {}:{:02}:{:02}.{:01}]}} ",
                hours, minutes, secs, tenth_secs
            );

            // Next position.turn() returns the next player to move, not the player that made
            // the current move m
            if position.turn() == Color::White {
                pgn.push_str(&counter.to_string());
                pgn.push_str("... ");
                pgn.push_str(&m.unwrap());
                pgn.push_str(&clock_comment);
                counter += 1;
            } else {
                pgn.push_str(&counter.to_string());
                pgn.push_str(". ");
                pgn.push_str(&m.unwrap());
                pgn.push_str(&clock_comment);
            }
        }

        pgn.push_str(&self.game.pgn_headers.result);

        String::from(pgn)
    }

    fn white(&mut self) -> Self::PlayerType {
        match self.players.top.color.as_str() {
            "white" => self.players.top.clone(),
            _ => self.players.bottom.clone(),
        }
    }

    fn black(&mut self) -> Self::PlayerType {
        match self.players.top.color.as_str() {
            "black" => self.players.top.clone(),
            _ => self.players.bottom.clone(),
        }
    }

    fn url(&self) -> String {
        format!("https://www.chess.com/live/game/{}", self.game.id)
    }

    fn end_time(&self) -> DateTime<Utc> {
        self.game.end_time.clone()
    }
}

/// Turn a chess.com timestamp into hours, minutes, seconds, and tenths of a second
fn time_from_timestamp(ts: u32) -> (u32, u32, u32, u32) {
    let tenth_secs = ts % 10;
    let mut secs = ts / 10;
    let mut minutes = secs / 60;
    let hours = minutes / 60;

    secs -= minutes * 60;
    minutes -= hours * 60;

    (hours, minutes, secs, tenth_secs)
}

impl DisplayableChessGame for CallbackLiveGame {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_from_timestamp() {
        let timestamp = 599;
        let (hours, minutes, secs, tenth_secs) = time_from_timestamp(timestamp);

        assert_eq!(hours, 0);
        assert_eq!(minutes, 0);
        assert_eq!(secs, 59);
        assert_eq!(tenth_secs, 9);

        let timestamp = 1800;
        let (hours, minutes, secs, tenth_secs) = time_from_timestamp(timestamp);

        assert_eq!(hours, 0);
        assert_eq!(minutes, 3);
        assert_eq!(secs, 0);
        assert_eq!(tenth_secs, 0);

        let timestamp = 1086;
        let (hours, minutes, secs, tenth_secs) = time_from_timestamp(timestamp);

        assert_eq!(hours, 0);
        assert_eq!(minutes, 1);
        assert_eq!(secs, 48);
        assert_eq!(tenth_secs, 6);
    }
}
