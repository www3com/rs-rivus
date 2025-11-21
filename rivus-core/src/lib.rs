use serde::Serialize;

// 封装返回结果
#[derive(Serialize)]
pub struct R<T: Serialize> {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> R<T> {
    pub fn ok(message: String, data: T) -> Self {
        Self {
            code: Code::Ok.as_i32(),
            message,
            data: Some(data),
        }
    }
    pub fn err(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
        }
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

// 封装返回结果
#[repr(i32)]
#[derive(Copy, Clone)]
pub enum Code {
    Ok = 200,
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    InternalServerError = 500,
}

impl Code {
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }
    pub fn to_string(&self) -> String {
        self.as_i32().to_string()
    }
}

#[test]
fn test_code() {
    assert_eq!(Code::Ok.as_i32(), 200);
    assert_eq!(Code::BadRequest.as_i32(), 400);
}
