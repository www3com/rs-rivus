use std::collections::HashMap;
use serde::Serialize;
use crate::code::Code;

#[derive(Serialize)]
pub struct R<T: Serialize> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
    pub args: Option<HashMap<&'static str, String>>,
}

impl<T: Serialize> R<T> {
    pub fn ok(data: T) -> Self {
        Self {
            code: Code::Ok.as_i32(),
            message: "ok".to_string(),
            data: Some(data),
            args: None,
        }
    }

    pub fn ok_with_message(data: T, message: String) -> Self {
        Self {
            code: Code::Ok.as_i32(),
            message,
            data: Some(data),
            args: None,
        }
    }

    pub fn err(code: i32) -> Self {
        Self {
            code,
            message: "error".to_string(),
            data: None,
            args: None,
        }
    }

    pub fn err_with_message(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
            args: None,
        }
    }

    pub fn err_with_args(code: i32, args: HashMap<&'static str, String>) -> Self {
        Self {
            code,
            message: "error".to_string(),
            data: None,
            args: Some(args),
        }
    }
}
