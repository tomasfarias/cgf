use std::fmt;

use prettytable::Table;

use crate::api::{ChessPlayer, DisplayableChessGame};
use crate::error::ChessError;

pub enum GameDisplayer {
    Default(String),
    Table(Table),
}

impl GameDisplayer {
    pub fn from_str(
        game: &mut impl DisplayableChessGame,
        output: &str,
    ) -> Result<Self, ChessError> {
        match output {
            "json" => match game.to_json() {
                Ok(json) => Ok(GameDisplayer::Default(json)),
                Err(e) => Err(ChessError::JSONError(e)),
            },
            "json-pretty" => match game.to_json_pretty() {
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
                    format!("{} ({}) ♔", white.name(), white.rating()),
                    format!("{} ({}) ♚", black.name(), black.rating()),
                ]);

                if white.result().is_some() && black.result().is_some() {
                    game_table.add_row(row![
                        "Result",
                        // Safe to unwrap as we have checked for is_some
                        format!("{}", white.result().unwrap()),
                        format!("{}", black.result().unwrap()),
                    ]);
                }

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
