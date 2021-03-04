use std::fmt;

use prettytable::Table;
use serde_json;

use crate::api::Game;
use crate::error::ChessError;

pub enum GameDisplayer {
    Default(String),
    Table(Table),
}

impl GameDisplayer {
    pub fn from_str(game: Game, output: &str) -> Result<Self, ChessError> {
        match output {
            "json" => match serde_json::to_string(&game) {
                Ok(json) => Ok(GameDisplayer::Default(json)),
                Err(e) => Err(ChessError::JSONError(e)),
            },
            "json-pretty" => match serde_json::to_string_pretty(&game) {
                Ok(json) => Ok(GameDisplayer::Default(json)),
                Err(e) => Err(ChessError::JSONError(e)),
            },
            "pgn" => Ok(GameDisplayer::Default(game.pgn)),
            "table" => {
                let mut game_table = Table::new();
                game_table.add_row(row![
                    "Players",
                    format!("{} ({}) ♔", game.white.username, game.white.rating),
                    format!("{} ({}) ♚", game.black.username, game.black.rating),
                ]);

                game_table.add_row(row![
                    "Result",
                    format!("{}", game.white.result),
                    format!("{}", game.black.result),
                ]);

                game_table.add_row(row![
                    "URL",
                    H2 -> game.url,
                ]);
                Ok(GameDisplayer::Table(game_table))
            }
            out => {
                return Err(ChessError::UnsupportedOutputError(out.to_string()));
            }
        }
    }
}

impl fmt::Display for GameDisplayer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameDisplayer::Default(s) => write!(f, "{}", s),
            GameDisplayer::Table(t) => write!(f, "{}", t),
        }
    }
}
