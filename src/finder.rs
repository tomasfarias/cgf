use log;

use chrono::{self, DateTime, Datelike, Utc};
use reqwest::Url;

use crate::api::{CallbackLiveGame, ChessApiClient, Game};
use crate::error::ChessError;

pub enum Pieces {
    Black,
    White,
}
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

pub struct GameFinder {
    pub search: Search,
    pieces: Option<Pieces>,
    year: Option<u32>,
    month: Option<u32>,
    day: Option<u32>,
    opponent: Option<String>,
}

impl GameFinder {
    pub fn by_player(player: &str) -> Self {
        GameFinder {
            search: Search::Player(player.to_owned()),
            pieces: None,
            year: None,
            month: None,
            day: None,
            opponent: None,
        }
    }

    pub fn by_id(id: &str) -> Self {
        GameFinder {
            search: Search::ID(id.to_owned()),
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

    pub fn find_by_id(&self) -> Result<CallbackLiveGame, ChessError> {
        let api = ChessApiClient::new(10)?;
        let id = self.search.get_value();
        log::info!("Getting game by id");
        Ok(api.get_live_game(&id)?)
    }

    pub fn find_by_player(&self) -> Result<Game, ChessError> {
        let api = ChessApiClient::new(10)?;
        let player = self.search.get_value();
        log::info!("Getting game archives");
        let game_archives = api.get_archives(&player)?;
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
            let mut games = api.get_month_games(&player, *year, *month)?;
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
                                    &g.black.username.to_lowercase() == player
                                        && &g.white.username.to_lowercase() == o
                                }
                                None => &g.black.username.to_lowercase() == player,
                            },
                            Pieces::White => match &self.opponent {
                                Some(o) => {
                                    &g.white.username.to_lowercase() == player
                                        && &g.black.username.to_lowercase() == o
                                }
                                None => &g.white.username.to_lowercase() == player,
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
