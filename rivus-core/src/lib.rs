mod code;

use serde::Serialize;
use crate::code::Code;

// 封装返回结果
#[derive(Serialize)]
pub struct R<T: Serialize> {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
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

    pub fn is_ok(&self) -> bool {
        self.code == Code::Ok.as_i32()
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

    pub fn is_err(&self) -> bool {
        self.code != Code::Ok.as_i32()
    }
}

/// 分页结果
#[derive(Serialize)]
pub struct Page<T: Serialize> {
    pub total: u64,
    pub items: Vec<T>,
}

impl<T: Serialize> Page<T> {
    pub fn new(total: u64, items: Vec<T>) -> Self {
        Self { total, items }
    }
}
