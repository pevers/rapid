use rapid_parser::{parse_module, ParseError};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::console;

#[derive(Serialize, Deserialize)]
struct ParseResult {
    success: bool,
    errors: Vec<ParseErrorInfo>,
}

#[derive(Serialize, Deserialize)]
struct ParseErrorInfo {
    message: String,
    error_position: Option<(usize, usize)>,
}

#[wasm_bindgen]
pub fn parse_rapid(input: &str) -> Result<JsValue, JsValue> {
    console::log_1(&"Parsing RAPID code...".into());

    let result = match parse_module(input) {
        Ok(_) => ParseResult {
            success: true,
            errors: vec![],
        },
        Err(errors) => {
            let parse_error = match errors {
                ParseError::UnrecognizedEof { location, expected } => ParseErrorInfo {
                    message: format!("Unexpected EOF. Expected one of: {}", expected.join(", ")),
                    error_position: Some((location, input.len())),
                },
                ParseError::UnrecognizedToken { token, expected } => ParseErrorInfo {
                    message: format!(
                        "Unexpected token '{}'. Expected one of: {}",
                        token.1,
                        expected.join(", ")
                    ),
                    error_position: Some((token.0, token.2)),
                },
                ParseError::InvalidToken { location } => ParseErrorInfo {
                    message: format!("Invalid token at location: {:?}", location),
                    error_position: Some((location, location)),
                },
                ParseError::ExtraToken { token } => ParseErrorInfo {
                    message: format!("Extra token at location: {:?}", token),
                    error_position: Some((token.0, token.2)),
                },
                ParseError::User { error } => ParseErrorInfo {
                    message: format!("User error: {}", error),
                    error_position: None,
                },
            };

            ParseResult {
                success: false,
                errors: vec![parse_error],
            }
        }
    };

    Ok(serde_json::to_string(&result).unwrap().into())
}
