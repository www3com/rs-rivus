use axum::Json;
use axum::body::Body;
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;
use validator::ValidationErrors;
use rivus_core::code::Code;
use rivus_core::r::R;

pub struct Rok<T>(pub T);

impl<T: Serialize> IntoResponse for Rok<T> {
    fn into_response(self) -> Response<Body> {
        let r = R::ok(Some(self.0));
        (StatusCode::OK, Json(r)).into_response()
    }
}

#[allow(dead_code)]
#[derive(Error)]
pub enum Rerr {
    #[error("{0}")]
    Of(i32),
    #[error("{0}")]
    OfMessage(i32, HashMap<&'static str, String>),
    #[error("{0}")]
    Validate(#[from] ValidationErrors),
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl fmt::Debug for Rerr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rerr::Of(code) => write!(f, "Error({:?})", code),
            Rerr::OfMessage(code, params) => write!(f, "Error({:?}, {:?})", code, params),
            Rerr::Validate(err) => write!(f, "ValidationError({:?})", err),
            Rerr::Other(err) => write!(f, "{:?}", err),
        }
    }
}

impl IntoResponse for Rerr {
    fn into_response(self) -> Response<Body> {
        let (status, r) = match self {
            Rerr::Of(code) => {
                (
                    StatusCode::OK,
                    R::<()>::err(code),
                )
            }
            Rerr::OfMessage(code, params) => (
                StatusCode::OK,
                R::err_with_args(code, params),
            ),
            Rerr::Validate(e) => (
                StatusCode::BAD_REQUEST,
                R::err_with_message(Code::BadRequest.as_i32(), e.to_string()),
            ),
            Rerr::Other(_) => {
                tracing::error!("{:?}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    R::err(Code::InternalServerError.as_i32()),
                )
            }
        };

        (status, Json(r)).into_response()
    }
}