use std::fmt;

use prettytable::Table;
use serde::ser::Serialize;
use serde_json;

use crate::client::ChessGame;
use crate::error::ChessError;

pub enum GameDisplayer {
    Default(String),
    Table(Table),
}

impl GameDisplayer {
    pub fn from_str(
        game: &mut (impl ChessGame + Serialize),
        output: &str,
    ) -> Result<Self, ChessError> {
        match output {
            "json" => match serde_json::to_string(&game) {
                Ok(json) => Ok(GameDisplayer::Default(json)),
                Err(e) => Err(ChessError::JSONError(e)),
            },
            "json-pretty" => match serde_json::to_string_pretty(&game) {
                Ok(json) => Ok(GameDisplayer::Default(json)),
                Err(e) => Err(ChessError::JSONError(e)),
            },
            "pgn" => Ok(GameDisplayer::Default(game.pgn().to_string())),
            "table" => {
                let mut game_table = Table::new();
                let white = game.white();
                let black = game.black();

                game_table.add_row(row![
                    "Players",
                    format!("{} ({}) ♔", white.username, white.rating),
                    format!("{} ({}) ♚", black.username, black.rating),
                ]);

                game_table.add_row(row![
                    "Result",
                    format!("{}", white.result),
                    format!("{}", black.result),
                ]);

                game_table.add_row(row![
                    "URL",
                    H2 -> game.url(),
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
