use log;

use chrono::{self, DateTime, Datelike, Utc};
use reqwest::Url;

use crate::api::{
    chessdotcom::GameArchives, ChessGame, ChessPlayer, DisplayableChessGame, Game, Games,
};
use crate::client::ChessClient;
use crate::error::ChessError;

#[derive(PartialEq, Debug)]
pub enum Pieces {
    Black,
    White,
}

#[derive(PartialEq, Debug)]
pub enum Search {
    Player(String),
    ID(String),
}

impl Search {
    pub fn get_value(&self) -> &String {
        match self {
            Search::Player(s) => s,
            Search::ID(s) => s,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct GameFinder {
    pub search: Search,
    pub api: String,
    pub pieces: Option<Pieces>,
    pub year: Option<u32>,
    pub month: Option<u32>,
    pub day: Option<u32>,
    pub opponent: Option<String>,
}

impl GameFinder {
    pub fn by_player(player: &str, api: &str) -> Self {
        GameFinder {
            search: Search::Player(player.to_owned()),
            api: api.to_owned(),
            pieces: None,
            year: None,
            month: None,
            day: None,
            opponent: None,
        }
    }

    pub fn by_id(id: &str, api: &str) -> Self {
        GameFinder {
            search: Search::ID(id.to_owned()),
            api: api.to_owned(),
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

    pub fn find_by_id(&self) -> Result<Game, ChessError> {
        let client = ChessClient::new(10, &self.api)?;
        let id = self.search.get_value();
        log::info!("Getting game by id");
        let game = client.get_game(&id)?;
        Ok(game)
    }

    pub fn find_by_player(&self) -> Result<Game, ChessError> {
        let client = ChessClient::new(10, &self.api)?;
        let player = self.search.get_value();
        match self.api.as_str() {
            "chess.com" => {
                log::info!("Getting game archives");
                let game_archives = client.get_user_game_archives(&player)?;
                let archives: Vec<(u32, u32)> = self.year_month_archives(game_archives);

                log::info!("Looking for game, iterating through archives.");
                for date in archives.iter() {
                    let (year, month) = date;
                    log::info!("At {:?}/{:?}", month, year);

                    match client.get_user_month_games(&player, *year as i32, *month)? {
                        Games::ChessDotCom(mut v) => {
                            v.sort_by_key(|g| g.end_time());
                            for mut game in v.into_iter() {
                                if self.check_game_found(&mut game) {
                                    return Ok(Game::ChessDotCom(game));
                                }
                            }
                        }
                        _ => panic!("Should never happen"),
                    }
                }
            }
            "lichess.org" => {
                log::info!("Getting user games");
                let game = client.get_last_user_game(&player)?;
                return Ok(game);
            }
            a => panic!("Unsupported API: {}", a),
        };

        Err(ChessError::GameNotFoundError)
    }

    fn year_month_archives(&self, game_archives: GameArchives) -> Vec<(u32, u32)> {
        let archives = game_archives
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

        archives
    }

    fn check_game_found(&self, g: &mut impl DisplayableChessGame) -> bool {
        self.players_had_correct_colors(g) && self.played_on_expected_day(g)
    }

    fn played_on_expected_day(&self, g: &mut impl DisplayableChessGame) -> bool {
        match self.day {
            Some(d) => g.end_time().day() == d,
            None => true,
        }
    }

    fn players_had_correct_colors(&self, g: &mut impl DisplayableChessGame) -> bool {
        let player = self.search.get_value();

        match &self.pieces {
            Some(pieces) => match pieces {
                Pieces::Black => match &self.opponent {
                    Some(o) => {
                        &g.black().name().to_lowercase() == player
                            && &g.white().name().to_lowercase() == o
                    }
                    None => &g.black().name().to_lowercase() == player,
                },
                Pieces::White => match &self.opponent {
                    Some(o) => {
                        &g.white().name().to_lowercase() == player
                            && &g.black().name().to_lowercase() == o
                    }
                    None => &g.white().name().to_lowercase() == player,
                },
            },
            None => true,
        }
    }
}
