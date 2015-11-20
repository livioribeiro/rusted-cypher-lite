use std::error::Error;
use std::fmt;
use std::string::FromUtf8Error;
use hyper;
use rustc_serialize::json;
use time;
use url;


#[derive(Clone, Debug, PartialEq, RustcDecodable)]
pub struct Neo4jError {
    pub message: String,
    pub code: String,
}

#[derive(Debug)]
pub struct GraphError {
    message: String,
    neo4j_errors: Option<Vec<Neo4jError>>,
    cause: Option<Box<Error>>,
}

impl GraphError {
    pub fn new(message: &str) -> Self {
        GraphError {
            message: message.to_owned(),
            neo4j_errors: None,
            cause: None,
        }
    }

    pub fn new_neo4j_error(errors: Vec<Neo4jError>) -> Self {
        GraphError {
            message: "Neo4j Error".to_owned(),
            neo4j_errors: Some(errors),
            cause: None,
        }
    }

    pub fn new_error(error: Box<Error>) -> Self {
        GraphError {
            message: "".to_owned(),
            neo4j_errors: None,
            cause: Some(error),
        }
    }
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for GraphError {
    fn description(&self) -> &str {
        match self.cause {
            Some(ref cause) => cause.description(),
            None => &self.message
        }
    }

    fn cause(&self) -> Option<&Error> {
        match self.cause {
            None => None,
            Some(ref e) => Some(&**e)
        }
    }
}

impl From<FromUtf8Error> for GraphError {
    fn from(error: FromUtf8Error) -> Self {
        GraphError {
            message: "FromUtf8Error".to_owned(),
            neo4j_errors: None,
            cause: Some(Box::new(error)),
        }
    }
}

impl From<url::ParseError> for GraphError {
    fn from(error: url::ParseError) -> Self {
        GraphError {
            message: "url::ParseError".to_owned(),
            neo4j_errors: None,
            cause: Some(Box::new(error)),
        }
    }
}

impl From<hyper::error::Error> for GraphError {
    fn from(error: hyper::error::Error) -> Self {
        GraphError {
            message: "hyper::error::Error".to_owned(),
            neo4j_errors: None,
            cause: Some(Box::new(error)),
        }
    }
}

impl From<json::DecoderError> for GraphError {
    fn from(error: json::DecoderError) -> Self {
        GraphError {
            message: "rustc_serialize::json::DecoderError".to_owned(),
            neo4j_errors: None,
            cause: Some(Box::new(error))
        }
    }
}

#[derive(Debug)]
pub struct TimeParseError(time::ParseError, String);

impl fmt::Display for TimeParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for TimeParseError {
    fn description(&self) -> &str {
        &self.1
    }
}

impl From<time::ParseError> for GraphError {
    fn from(error: time::ParseError) -> Self {
        GraphError {
            message: "time::ParseError".to_owned(),
            neo4j_errors: None,
            cause: Some(Box::new(TimeParseError(error, format!("{}", error)))),
        }
    }
}
