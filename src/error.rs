use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use tracing::error;

// The `Error` derive makes it easier to define errors by providing attributes like `error` to
// provide descriptions of errors and helpers like `#[from]` which make it really easy to create
// conversions from other errors into this one.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Not Found")]
    NotFound,

    #[error("Unathorized")]
    Unathorized,

    // This is returned in hivefriends when a received field is too long.
    #[error("{field} should not be longer than {maximum_length} characters")]
    TooManyCharacters {
        field: &'static str,
        maximum_length: u64,
    },

    #[error("A user with that name already exists")]
    UserExists,

    // This is returned in quotes when a received field is too short (empty).
    #[error("The field {0} is empty")]
    EmptyField(&'static str),

    // This is returned in quotes when a received array element is too short (empty).
    #[error("An element in {0} is empty")]
    EmptyArrayElement(&'static str),

    // This is returned in quotes when a received field within an array element is too short (empty).
    // Example:
    // return Err(Error::EmptyArrayField {
    //     array: "fragments",
    //     field: "content",
    // });
    #[error("{field} in {array} is empty")]
    EmptyArrayField {
        array: &'static str,
        field: &'static str,
    },

    #[error("Internal Server Error")]
    InternalError(#[from] anyhow::Error),

    #[error("{0}")]
    JsonRejection(#[from] JsonRejection),
}

// This is where we define what axum (web framework) should actually do with the error.
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        // First we figure out which status code the error correlates to.
        let status = match &self {
            Error::Unathorized => StatusCode::UNAUTHORIZED,
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::InternalError(e) => {
                // In the case of an internal error we won't return any information to the front
                // end so we log it instead so that we don't lose that information.
                // Since errors can contain further errors we iterate over those with the help of
                // the `chain()` method and join them all into one `String`.
                let err = e
                    .chain()
                    .skip(1)
                    .fold(e.to_string(), |acc, cause| format!("{}: {}\n", acc, cause));
                error!("API encountered error: {}", err);

                StatusCode::INTERNAL_SERVER_ERROR
            }
            Error::TooManyCharacters { .. }
            | Error::UserExists
            | Error::JsonRejection(_)
            | Error::EmptyField(_)
            | Error::EmptyArrayElement(_)
            | Error::EmptyArrayField { .. } => StatusCode::BAD_REQUEST,
        };

        // At some point I noticed that the errors were quite bad when invalid JSON was sent in so
        // I made sure to unwrap the actual error from the useless wrappers around it and return
        // that instead. This could possibly be improved but it's already more helpful to the API
        // consumper like this.
        let message = if let Error::JsonRejection(rej) = self {
            use std::error::Error;
            match rej {
                JsonRejection::JsonDataError(e) => e.source().unwrap().to_string(),
                JsonRejection::JsonSyntaxError(e) => e.source().unwrap().to_string(),
                _ => rej.to_string(),
            }
        } else {
            // Any other type of error gets to use its `Display` implementation instead which is
            // defined by the `error` attributes within the `Error` type.
            self.to_string()
        };

        // We contruct the actual body of the response.
        let body = Json(json!({
            "message": message,
        }));

        // The combination of status code and body is our response.
        (status, body).into_response()
    }
}
