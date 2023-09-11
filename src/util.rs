use std::{env, fmt};
use axum::http;
use dotenvy::dotenv;

pub fn load_environment_variable(variable_name: &str) -> String {
    dotenv().ok();
    env::var(variable_name)
        .expect(&format!("{} must be set", variable_name))
}

#[derive(Debug, PartialEq)]
pub enum ErrorType {
    NotFound,
    Internal,
    UniqueViolation,
}

#[derive(Debug)]
pub struct CustomError {
    pub err_type: ErrorType,
    pub message: String,
}

impl CustomError {
    pub fn new(message: &str, err_type: ErrorType) -> CustomError {
        CustomError { message: message.to_string(), err_type }
    }

    pub fn to_http_status(&self) -> http::StatusCode {
        match self.err_type {
            ErrorType::NotFound =>  http::StatusCode::NOT_FOUND,
            ErrorType::Internal => http::StatusCode::INTERNAL_SERVER_ERROR,
            ErrorType::UniqueViolation => http::StatusCode::BAD_REQUEST,
        }
    }

    pub fn from_diesel_err(err: diesel::result::Error, context: &str) -> CustomError {
        CustomError::new(
            format!("{}: {}", context, err.to_string()).as_str(),
            match err {
                diesel::result::Error::DatabaseError(db_err, _) => {
                    match db_err {
                        diesel::result::DatabaseErrorKind::UniqueViolation => ErrorType::UniqueViolation,
                        _ => ErrorType::Internal,
                    }
                }
                diesel::result::Error::NotFound => ErrorType::NotFound,
                // Here we can handle other cases if needed
                _ => {
                    ErrorType::Internal
                }
            },
        )
    }
}


impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
