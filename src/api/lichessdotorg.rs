use std::fmt::Debug;

use super::{ChessGame, ChessPlayer, DisplayableChessGame};
use chrono::serde::ts_seconds::deserialize as from_ts;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Game {
    pub id: String,
    pub rated: bool,
    pub variant: String,
    pub speed: String,
    pub perf: String,
    #[serde(deserialize_with = "from_ts")]
    pub created_at: DateTime<Utc>,
    #[serde(deserialize_with = "from_ts")]
    pub last_move_at: DateTime<Utc>,
    pub status: String,
    pub players: Players,
    pub opening: Option<Opening>,
    pub pgn: String,
    pub clock: Clock,
    pub moves: String,
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
        self.players.white.clone()
    }

    fn black(&mut self) -> Self::PlayerType {
        self.players.black.clone()
    }

    fn url(&self) -> String {
        format!("https://lichess.org/{}", self.id)
    }

    fn end_time(&self) -> DateTime<Utc> {
        self.last_move_at.clone()
    }
}

impl DisplayableChessGame for Game {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Players {
    pub white: Player,
    pub black: Player,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Player {
    // Fields may be missing if user is anonymous
    pub user: Option<User>,
    pub rating: Option<u32>,
    pub rating_diff: Option<i32>,
}

impl ChessPlayer for Player {
    fn name(&self) -> String {
        match &self.user {
            Some(u) => u.name.clone(),
            None => "Anonymous".to_string(),
        }
    }

    fn title(&self) -> Option<String> {
        if let Some(u) = &self.user {
            match &u.title {
                Some(t) => Some(t.clone()),
                None => None,
            }
        } else {
            None
        }
    }

    fn rating(&self) -> Option<u32> {
        if let Some(r) = self.rating {
            Some(r.clone())
        } else {
            None
        }
    }

    fn url(&self) -> Option<String> {
        if let Some(u) = &self.user {
            Some(format!("https://lichess.org/@/{}", u.id))
        } else {
            None
        }
    }

    fn result(&self) -> Option<String> {
        None
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub name: String,
    pub title: Option<String>,
    pub patron: Option<bool>,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Opening {
    pub eco: String,
    pub name: String,
    pub ply: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Clock {
    pub initial: u32,
    pub increment: u32,
    pub total_time: u32,
}
