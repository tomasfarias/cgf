use clap::{App, Arg};
use std::ffi::OsString;

use chrono::{DateTime, Utc};

use crate::displayer::GameDisplayer;
use crate::error::ChessError;
use crate::finder::GameFinder;

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

        Ok(ChessGameFinderCLI {
            output: output.to_owned(),
            finder: game_finder,
        })
    }

    pub fn run(self) -> Result<(), ChessError> {
        log::info!("Finding game");
        let game = self.finder.find()?;
        let displayer = GameDisplayer::from_str(game, &self.output)?;
        println!("{}", displayer);

        log::info!("Done!");
        Ok(())
    }
}
