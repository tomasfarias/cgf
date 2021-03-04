use std::error;
use std::fmt;

use reqwest;
use serde_json;

#[derive(Debug)]
pub enum ChessError {
    GameNotFoundError,
    UnsupportedOutputError(String),
    RequestError(reqwest::Error),
    JSONError(serde_json::Error),
}

impl fmt::Display for ChessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChessError::GameNotFoundError => {
                write!(f, "no game found that matches requested parameters")
            }
            ChessError::RequestError(..) => write!(f, "a request to the chess api failed"),
            ChessError::JSONError(..) => {
                write!(f, "JSON game serialization or deserialization failed")
            }
            ChessError::UnsupportedOutputError(out) => write!(f, "{} output is not supported", out),
        }
    }
}

impl error::Error for ChessError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            ChessError::GameNotFoundError => None,
            ChessError::UnsupportedOutputError(_) => None,
            ChessError::JSONError(ref e) => Some(e),
            ChessError::RequestError(ref e) => Some(e),
        }
    }
}

impl From<reqwest::Error> for ChessError {
    fn from(err: reqwest::Error) -> ChessError {
        ChessError::RequestError(err)
    }
}

impl From<serde_json::Error> for ChessError {
    fn from(err: serde_json::Error) -> ChessError {
        ChessError::JSONError(err)
    }
}
