use crate::code::Code;
use crate::r::R;
use axum::Json;
use axum::body::Body;
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use rust_i18n::t;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;
use validator::ValidationErrors;

pub struct Rok<T>(pub T);

impl<T: Serialize> IntoResponse for Rok<T> {
    fn into_response(self) -> Response<Body> {
        let message = t!(Code::Ok.to_string());
        let r = R::ok_with_message(Some(self.0), message.to_string());
        (StatusCode::OK, Json(r)).into_response()
    }
}

#[allow(dead_code)]
#[derive(Error)]
pub enum Rerr {
    #[error("{0}")]
    Of(i32),
    #[error("{0}")]
    OfMsg(i32, HashMap<&'static str, String>),
    #[error("{0}")]
    Validate(#[from] ValidationErrors),
    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

impl fmt::Debug for Rerr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rerr::Of(code) => write!(f, "Error({:?})", code),
            Rerr::OfMsg(code, params) => write!(f, "Error({:?}, {:?})", code, params),
            Rerr::Validate(err) => write!(f, "ValidationError({:?})", err),
            Rerr::Other(err) => write!(f, "{:?}", err),
        }
    }
}

impl IntoResponse for Rerr {
    fn into_response(self) -> Response<Body> {
        let (status, r) = match self {
            Rerr::Of(code) => {
                let message = t!(code.to_string());
                (
                    StatusCode::OK,
                    R::<()>::err_with_message(code, message.to_string()),
                )
            }
            Rerr::OfMsg(code, params) => (
                StatusCode::OK,
                R::err_with_message(code, format(code, params)),
            ),
            Rerr::Validate(e) => (
                StatusCode::BAD_REQUEST,
                R::err_with_message(Code::BadRequest.as_i32(), e.to_string()),
            ),
            Rerr::Other(_) => {
                tracing::error!("{:?}", self);
                let message = t!(Code::InternalServerError.to_string());
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    R::err_with_message(Code::InternalServerError.as_i32(), message.to_string()),
                )
            }
        };

        (status, Json(r)).into_response()
    }
}

fn format(code: i32, params: HashMap<&'static str, String>) -> String {
    let mut template = t!(code.to_string()).to_string();
    for (key, value) in params {
        template = template.replace(&format!("{{{{{}}}}}", key), value.as_str());
    }
    template
}
