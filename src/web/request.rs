use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Deserialize, Serialize, Debug)]
pub struct RequestBody {
    pub payload: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ResponseResult<'a> {
    payload: Option<Vec<JsonValue>>,
    error: Option<&'a str>,
}

impl<'a> ResponseResult<'a> {
    pub fn new() -> ResponseResult<'a> {
        ResponseResult {
            payload: None,
            error: None,
        }
    }

    pub fn payload(self, payload: Vec<JsonValue>) -> Self {
        ResponseResult {
            payload: Some(payload),
            ..self
        }
    }

    pub fn error(self, error: &'a str) -> Self {
        ResponseResult {
            error: Some(error),
            ..self
        }
    }
}
