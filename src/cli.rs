use clap::{App, Arg, ArgGroup};
use std::ffi::OsString;

use chrono::{DateTime, Utc};

use crate::displayer::GameDisplayer;
use crate::error::ChessError;
use crate::finder::{GameFinder, Search};

pub struct ChessGameFinderCLI {
    output: String,
    finder: GameFinder,
}

impl ChessGameFinderCLI {
    pub fn new() -> Self {
        Self::new_from(std::env::args_os().into_iter()).unwrap_or_else(|e| e.exit())
    }

    pub fn new_from<I, T>(args: I) -> Result<Self, clap::Error>
    where
        I: Iterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let displays = &["pgn", "json-pretty", "json"];

        let app = App::new("Chess game finder")
        .version("0.3.2")
        .author("Tomas Farias <tomas@tomasfarias.dev>")
        .about("Finds games using online chess APIs")
        .arg(
            Arg::with_name("player_or_id")
                .takes_value(true)
                .required(true)
                .value_name("PLAYER_OR_ID")
                .help("A Game ID or a player's username whose game to look for. If it contains all digits, will assume it's a Game ID unless the --player flag is used."),
        )
        .arg(
            Arg::with_name("player")
                .takes_value(false)
                .long("player")
                .help("Force search by player username instead game ID."),
        )
        .arg(
            Arg::with_name("api")
                .long("api")
                .short("a")
                .takes_value(true)
                .default_value("chess.com")
                .possible_values(&["chess.com", "lichess.org"])
                .required(false)
                .help("Choose the API where to find your chess games."),
        )
        .arg(
            Arg::with_name("white")
                .long("white")
                .takes_value(false)
                .conflicts_with("black")
                .help("Fetch games with white pieces. Cannot be used simultaneously with --black."),
        )
        .arg(
            Arg::with_name("black")
                .long("black")
                .takes_value(false)
                .conflicts_with("white")
                .help("Fetch games with black pieces. Cannot be used simultaneously with --white."),
        )
        .arg(
            Arg::with_name("json")
                .long("json")
                .takes_value(false)
                .help("Output game as JSON"),
        )
        .arg(
            Arg::with_name("json-pretty")
                .long("json-pretty")
                .takes_value(false)
                .help("Output game as pretty JSON"),
        )
        .arg(
            Arg::with_name("pgn")
                .long("pgn")
                .takes_value(false)
                .help("Output game PGN string"),
        )
        .group(
            ArgGroup::with_name("display")
                .args(displays)
                .multiple(false)
                .required(false),
        )
        .arg(
            Arg::with_name("year")
                .short("y")
                .long("year")
                .takes_value(true)
                .conflicts_with("date")
                .help("Fetch games from a specific year"),
        )
        .arg(
            Arg::with_name("day")
                .short("d")
                .long("day")
                .takes_value(true)
                .conflicts_with("date")
                .help("Fetch games from a specific day of the month (1-31)"),
        )
        .arg(
            Arg::with_name("month")
                .short("m")
                .long("month")
                .takes_value(true)
                .conflicts_with("date")
                .help("Fetch games from a specific month (1-12)"),
        )
        .arg(
            Arg::with_name("date")
                .long("date")
                .takes_value(true)
                .help("Fetch games from a specific date in RFC-3339 format"),
        );

        let matches = app.get_matches_from_safe(args)?;

        let player_or_id = matches
            .value_of("player_or_id")
            .expect("player or id argument is required");
        let api = matches.value_of("api").expect("api defaults to chess.com");
        let mut game_finder =
            if matches.is_present("player") || !player_or_id.chars().all(char::is_numeric) {
                GameFinder::by_player(player_or_id, api)
            } else {
                GameFinder::by_id(player_or_id, api)
            };

        if matches.is_present("white") {
            game_finder.white();
        } else if matches.is_present("black") {
            game_finder.black();
        }

        if matches.is_present("date") {
            let date = matches.value_of("date").expect("date is present");
            let parsed_date = DateTime::parse_from_rfc3339(date)
                .unwrap()
                .with_timezone(&Utc);
            game_finder.date(parsed_date);
        }

        match matches.value_of("year") {
            Some(y) => {
                let year = y.parse::<u32>().unwrap();
                game_finder.year(year);
            }
            None => (),
        };

        match matches.value_of("month") {
            Some(m) => {
                let month = m.parse::<u32>().unwrap();
                game_finder.month(month);
            }
            None => (),
        };

        match matches.value_of("day") {
            Some(d) => {
                let day = d.parse::<u32>().unwrap();
                game_finder.day(day);
            }
            None => (),
        };

        let mut output = "table";

        for display in displays {
            if matches.is_present(display) {
                output = display;
                break;
            }
        }

        Ok(ChessGameFinderCLI {
            output: output.to_owned(),
            finder: game_finder,
        })
    }

    pub fn run(self) -> Result<(), ChessError> {
        log::info!("Finding game");
        match self.finder.search {
            Search::Player(_) => {
                let mut game = self.finder.find_by_player()?;
                let displayer = GameDisplayer::from_str(&mut game, &self.output)?;
                println!("{}", displayer);
            }
            Search::ID(_) => {
                let mut game = self.finder.find_by_id()?;
                let displayer = GameDisplayer::from_str(&mut game, &self.output)?;
                println!("{}", displayer);
            }
        }

        log::info!("Done!");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finder::Pieces;

    #[test]
    fn test_single_game_id() {
        let args = vec!["cgf", "12345678910"];
        let cgf = ChessGameFinderCLI::new_from(args.into_iter()).unwrap();
        let finder = GameFinder {
            search: Search::ID("12345678910".to_owned()),
            api: "chess.com".to_string(),
            pieces: None,
            year: None,
            month: None,
            day: None,
            opponent: None,
        };
        assert_eq!(cgf.finder, finder);
    }

    #[test]
    fn test_single_player_username() {
        let args = vec!["cgf", "a_player"];
        let cgf = ChessGameFinderCLI::new_from(args.into_iter()).unwrap();
        let finder = GameFinder {
            search: Search::Player("a_player".to_owned()),
            api: "chess.com".to_string(),
            pieces: None,
            year: None,
            month: None,
            day: None,
            opponent: None,
        };
        assert_eq!(cgf.finder, finder);
    }

    #[test]
    fn test_numeric_player_username() {
        let args = vec!["cgf", "12345678910", "--player"];
        let cgf = ChessGameFinderCLI::new_from(args.into_iter()).unwrap();
        let finder = GameFinder {
            search: Search::Player("12345678910".to_owned()),
            api: "chess.com".to_string(),
            pieces: None,
            year: None,
            month: None,
            day: None,
            opponent: None,
        };
        assert_eq!(cgf.finder, finder);
    }

    #[test]
    fn test_chess_dot_com_api_choice() {
        let args = vec!["cgf", "a_player", "--api=chess.com"];
        let cgf = ChessGameFinderCLI::new_from(args.into_iter()).unwrap();
        let finder = GameFinder {
            search: Search::Player("a_player".to_owned()),
            api: "chess.com".to_string(),
            pieces: None,
            year: None,
            month: None,
            day: None,
            opponent: None,
        };
        assert_eq!(cgf.finder, finder);
    }

    #[test]
    fn test_lichess_dot_org_api_choice() {
        let args = vec!["cgf", "a_player", "--api=lichess.org"];
        let cgf = ChessGameFinderCLI::new_from(args.into_iter()).unwrap();
        let finder = GameFinder {
            search: Search::Player("a_player".to_owned()),
            api: "lichess.org".to_string(),
            pieces: None,
            year: None,
            month: None,
            day: None,
            opponent: None,
        };
        assert_eq!(cgf.finder, finder);
    }

    #[test]
    fn test_white_player_username() {
        let args = vec!["cgf", "a_player", "--white"];
        let cgf = ChessGameFinderCLI::new_from(args.into_iter()).unwrap();
        let finder = GameFinder {
            search: Search::Player("a_player".to_owned()),
            api: "chess.com".to_string(),
            pieces: Some(Pieces::White),
            year: None,
            month: None,
            day: None,
            opponent: None,
        };
        assert_eq!(cgf.finder, finder);
    }

    #[test]
    fn test_black_player_username() {
        let args = vec!["cgf", "a_player", "--black"];
        let cgf = ChessGameFinderCLI::new_from(args.into_iter()).unwrap();
        let finder = GameFinder {
            search: Search::Player("a_player".to_owned()),
            api: "chess.com".to_string(),
            pieces: Some(Pieces::Black),
            year: None,
            month: None,
            day: None,
            opponent: None,
        };
        assert_eq!(cgf.finder, finder);
    }
}
