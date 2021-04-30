use http_api_problem::HttpApiProblem as Problem;
use std::convert::Infallible;
use warp::http;
use warp::{Rejection, Reply};

pub fn build<E: Into<anyhow::Error>>(err: E) -> Rejection {
    warp::reject::custom(pack(err.into()))
}

pub fn pack(err: anyhow::Error) -> Problem {
    let err = match err.downcast::<Problem>() {
        Ok(problem) => return problem,
        Err(err) => err,
    };

    tracing::error!(message = "internal error occurred", error = ?err);

    Problem::with_title_and_type(http::StatusCode::INTERNAL_SERVER_ERROR)
}

fn reply_from_problem(problem: &Problem) -> impl Reply {
    let code = problem
        .status
        .unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR);

    let reply = warp::reply::json(problem);
    let reply = warp::reply::with_status(reply, code);
    warp::reply::with_header(
        reply,
        http::header::CONTENT_TYPE,
        http_api_problem::PROBLEM_JSON_MEDIA_TYPE,
    )
}

pub async fn unpack(rejection: Rejection) -> Result<impl Reply, Infallible> {
    let reply = if rejection.is_not_found() {
        let problem = Problem::with_title_and_type(http::StatusCode::NOT_FOUND);
        reply_from_problem(&problem)
    } else if let Some(problem) = rejection.find::<Problem>() {
        reply_from_problem(problem)
    } else if let Some(e) = rejection.find::<warp::filters::body::BodyDeserializeError>() {
        let problem = Problem::new(http::StatusCode::BAD_REQUEST)
            .title("Invalid Request Body.")
            .detail(format!("Request body is invalid. {}", e));
        reply_from_problem(&problem)
    } else if rejection.find::<warp::reject::MethodNotAllowed>().is_some() {
        let problem = Problem::with_title_and_type(http::StatusCode::METHOD_NOT_ALLOWED);
        reply_from_problem(&problem)
    } else if rejection.find::<warp::reject::InvalidQuery>().is_some() {
        let problem = Problem::with_title_and_type(http::StatusCode::BAD_REQUEST);
        reply_from_problem(&problem)
    } else {
        tracing::error!(
            message = "unhandled rejection while unpacking rejection",
            ?rejection
        );
        let problem = Problem::with_title_and_type(http::StatusCode::INTERNAL_SERVER_ERROR);
        reply_from_problem(&problem)
    };

    Ok(reply)
}
