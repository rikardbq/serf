use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct RequestBody {
    pub payload: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ResponseResult<T> {
    payload: Option<T>,
    error: Option<T>,
}

impl<T> ResponseResult<T> {
    pub fn new() -> ResponseResult<T> {
        ResponseResult {
            payload: None,
            error: None,
        }
    }

    pub fn payload(self, payload: T) -> Self {
        ResponseResult {
            payload: Some(payload),
            ..self
        }
    }

    pub fn error(self, error: T) -> Self {
        ResponseResult {
            error: Some(error),
            ..self
        }
    }
}
