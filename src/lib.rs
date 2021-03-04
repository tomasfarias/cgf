use log;
use std::error;
use std::ffi::OsString;
use std::fmt;
use std::time::Duration;

use chrono::serde::ts_seconds::deserialize as from_ts;
use chrono::serde::ts_seconds_option::deserialize as from_ts_option;
use chrono::{self, DateTime, Datelike, Utc};
use clap::{App, Arg};
use reqwest::{self, Url};
use serde::{Deserialize, Serialize};
use serde_json;

#[macro_use]
extern crate prettytable;
use prettytable::Table;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    username: String,
    rating: u16,
    result: String,
    #[serde(rename(serialize = "id", deserialize = "@id"))]
    id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Game {
    white: Player,
    black: Player,
    url: String,
    fen: String,
    pub pgn: String,
    #[serde(deserialize_with = "from_ts_option")]
    #[serde(default)]
    start_time: Option<DateTime<Utc>>,
    #[serde(deserialize_with = "from_ts")]
    end_time: DateTime<Utc>,
    time_control: String,
    rules: String,
    eco: Option<String>,
    tournament: Option<String>,
    r#match: Option<String>,
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
    games: Vec<Game>,
}

#[derive(Deserialize, Debug)]
pub struct GameArchives {
    archives: Vec<String>,
}

pub struct ChessApi {
    client: reqwest::blocking::Client,
}

impl ChessApi {
    pub fn new(timeout: u64) -> Result<Self, reqwest::Error> {
        let timeout = Duration::new(timeout, 0);

        Ok(ChessApi {
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

pub enum Pieces {
    Black,
    White,
}

pub struct GameFinder {
    player: String,
    pieces: Option<Pieces>,
    year: Option<u32>,
    month: Option<u32>,
    day: Option<u32>,
    opponent: Option<String>,
}

impl GameFinder {
    pub fn new(player: &str) -> Self {
        GameFinder {
            player: player.to_owned(),
            pieces: None,
            year: None,
            month: None,
            day: None,
            opponent: None,
        }
    }

    pub fn white<'a>(&'a mut self) -> &'a mut GameFinder {
        self.pieces = Some(Pieces::White);
        self
    }

    pub fn black<'a>(&'a mut self) -> &'a mut GameFinder {
        self.pieces = Some(Pieces::Black);
        self
    }

    pub fn year<'a>(&'a mut self, year: u32) -> &'a mut GameFinder {
        self.year = Some(year);
        self
    }

    pub fn month<'a>(&'a mut self, month: u32) -> &'a mut GameFinder {
        self.month = Some(month);
        self
    }

    pub fn day<'a>(&'a mut self, day: u32) -> &'a mut GameFinder {
        self.day = Some(day);
        self
    }

    pub fn today<'a>(&'a mut self) -> &'a mut GameFinder {
        let utc: DateTime<Utc> = Utc::now();
        self.year = Some(utc.year() as u32);
        self.month = Some(utc.month());
        self.day = Some(utc.day());
        self
    }

    pub fn date<'a>(&'a mut self, date: DateTime<Utc>) -> &'a mut GameFinder {
        self.year = Some(date.year() as u32);
        self.month = Some(date.month());
        self.day = Some(date.day());
        self
    }

    pub fn oponent<'a>(&'a mut self, opponent: &str) -> &'a mut GameFinder {
        let mut opponent = opponent.to_owned();
        opponent.make_ascii_lowercase();
        self.opponent = Some(opponent);
        self
    }

    pub fn find(&self) -> Result<Game, ChessError> {
        let api = ChessApi::new(10)?;
        log::info!("Getting game archives");
        let game_archives = api.get_archives(&self.player)?;
        log::debug!("Archives: {:?}", game_archives);
        let mut archives: Vec<(u32, u32)> = game_archives
            .archives
            .iter()
            .map(|s| Url::parse(s))
            .filter_map(Result::ok)
            .map(|u| {
                let mut segments = u.path_segments().unwrap();
                segments.next();
                segments.next();
                segments.next();
                segments.next();

                let year = segments.next().unwrap().parse::<u32>().unwrap();
                let month = segments.next().unwrap().parse::<u32>().unwrap();

                (year, month)
            })
            .filter(|&(y, m)| match self.year {
                Some(year) => match self.month {
                    Some(month) => year == y && month == m,
                    None => year == y,
                },
                None => match self.month {
                    Some(month) => month == m,
                    None => true,
                },
            })
            .collect::<Vec<(u32, u32)>>();

        archives.reverse();

        log::info!("Looking for game, iterating through archives.");
        for date in archives.iter() {
            let (year, month) = date;
            log::info!("At {:?}/{:?}", month, year);
            let mut games = api.get_month_games(&self.player, *year, *month)?;
            log::debug!("Games: {:?}", games);
            games.sort_by_key(|g| g.end_time);

            let mut filtered = games
                .iter()
                .filter(|g| {
                    // Filter opponent's and user's piece color
                    match &self.pieces {
                        Some(pieces) => match pieces {
                            Pieces::Black => match &self.opponent {
                                Some(o) => {
                                    g.black.username.to_lowercase() == self.player
                                        && &g.white.username.to_lowercase() == o
                                }
                                None => g.black.username.to_lowercase() == self.player,
                            },
                            Pieces::White => match &self.opponent {
                                Some(o) => {
                                    g.white.username.to_lowercase() == self.player
                                        && &g.black.username.to_lowercase() == o
                                }
                                None => g.white.username.to_lowercase() == self.player,
                            },
                        },
                        None => true,
                    }
                })
                .filter(|g| match self.day {
                    Some(d) => g.end_time.day() == d,
                    None => true,
                })
                .cloned()
                .collect::<Vec<Game>>();

            log::debug!("Filtered games: {:?}", filtered);
            if !filtered.is_empty() {
                return Ok(filtered.pop().unwrap());
            };
        }

        Err(ChessError::GameNotFoundError)
    }
}

#[derive(Debug)]
pub enum ChessError {
    GameNotFoundError,
    UnsupportedOutputError(String),
    RequestError(reqwest::Error),
}

impl fmt::Display for ChessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChessError::GameNotFoundError => {
                write!(f, "no game found that matches requested parameters")
            }
            ChessError::RequestError(..) => write!(f, "a request to the chess api failed"),
            ChessError::UnsupportedOutputError(out) => write!(f, "{} output is not supported", out),
        }
    }
}

impl error::Error for ChessError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            ChessError::GameNotFoundError => None,
            ChessError::UnsupportedOutputError(_) => None,
            ChessError::RequestError(ref e) => Some(e),
        }
    }
}

impl From<reqwest::Error> for ChessError {
    fn from(err: reqwest::Error) -> ChessError {
        ChessError::RequestError(err)
    }
}

pub struct ChessGameFinder {
    output: String,
    finder: GameFinder,
}

impl ChessGameFinder {
    pub fn new() -> Self {
        Self::new_from(std::env::args_os().into_iter()).unwrap_or_else(|e| e.exit())
    }

    pub fn new_from<I, T>(args: I) -> Result<Self, clap::Error>
    where
        I: Iterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let app = App::new("Chess game finder")
        .version("0.1.0")
        .author("Tomas Farias <tomas@tomasfarias.dev>")
        .about("Finds games using online chess APIs")
        .arg(
            Arg::with_name("player")
                .takes_value(true)
                .required(true)
                .value_name("PLAYER")
                .help("The player whose games to fetch"),
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
            Arg::with_name("pgn")
                .long("pgn")
                .takes_value(false)
                .help("Output game PGN string")
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

        let player = matches
            .value_of("player")
            .expect("player argument is required");
        let mut game_finder = GameFinder::new(player);

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

        let output = if matches.is_present("json") {
            "json"
        } else {
            if matches.is_present("pgn") {
                "pgn"
            } else {
                "table"
            }
        };

        Ok(ChessGameFinder {
            output: output.to_owned(),
            finder: game_finder,
        })
    }

    pub fn run(self) -> Result<(), ChessError> {
        log::info!("Finding game");
        let game = self.finder.find()?;

        match &self.output[..] {
            "json" => {
                println!(
                    "{}",
                    serde_json::to_string(&game).expect("JSON serialization error")
                );
            }
            "pgn" => {
                println!("{}", game.pgn);
            }
            "table" => {
                println!("{}", game);
            }
            out => {
                return Err(ChessError::UnsupportedOutputError(out.to_string()));
            }
        }

        log::info!("Done!");

        Ok(())
    }
}
