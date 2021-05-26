use std::collections::HashMap;

use serde::de::DeserializeOwned;
use validator::{Validate, ValidationErrors, ValidationErrorsKind};
use warp::{Filter, Rejection};

#[derive(Debug)]
pub(crate) struct Error(ValidationErrors);

impl Error {
    pub fn errors(&self) -> &HashMap<&'static str, ValidationErrorsKind> {
        &self.0.errors()
    }
}

impl warp::reject::Reject for Error {}

fn validate<T>(value: T) -> Result<T, Error>
where
    T: Validate,
{
    value.validate().map_err(Error)?;

    Ok(value)
}

pub(crate) fn with_validated_json<T>() -> impl Filter<Extract = (T,), Error = Rejection> + Clone
where
    T: DeserializeOwned + Validate + Send,
{
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
        .and_then(|value| async move { validate(value).map_err(warp::reject::custom) })
}
