use serde::Serialize;
use crate::code::Code;

#[derive(Serialize)]
pub struct R<T: Serialize> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

impl<T: Serialize> R<T> {
    pub fn ok(data: T) -> Self {
        Self {
            code: Code::Ok.as_i32(),
            message: "ok".to_string(),
            data: Some(data),
        }
    }

    pub fn ok_with_message(data: T, message: String) -> Self {
        Self {
            code: Code::Ok.as_i32(),
            message,
            data: Some(data),
        }
    }

    pub fn err(code: i32) -> Self {
        Self {
            code,
            message: "error".to_string(),
            data: None,
        }
    }

    pub fn err_with_message(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
        }
    }
}
