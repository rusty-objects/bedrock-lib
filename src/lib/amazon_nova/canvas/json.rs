//! Specific implementation of InvokeModel request/response structs Amazon Canvas model
//!
//! The rust structs here are set up so that serde generates compatible json according
//! to the published request and response schemas:
//!
//! https://docs.aws.amazon.com/nova/latest/userguide/image-gen-req-resp-structure.html

use std::fmt::Display;

// Had to do some serde field name changes in the types below to match the schema.
//
// https://serde.rs/field-attrs.html
// https://serde.rs/variant-attrs.html
// https://serde.rs/attr-skip-serializing.html
//
// https://stackoverflow.com/questions/59167416/how-can-i-deserialize-an-enum-when-the-case-doesnt-match
// https://stackoverflow.com/questions/53900612/how-do-i-avoid-generating-json-when-serializing-a-value-that-is-null-or-a-defaul
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CanvasRequest {
    pub task_type: String,
    pub text_to_image_params: TextToImageParams,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_generation_config: Option<ImageGenerationConfig>,
}
impl Display for CanvasRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let json = serde_json::to_string(&self).unwrap();
        f.write_str(json.as_str())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextToImageParams {
    pub text: String,

    #[serde(rename = "negativeText", skip_serializing_if = "String::is_empty")]
    pub negative_text: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ImageGenerationConfig;

#[derive(Serialize, Deserialize, Debug)]
pub struct CanvasResponse {
    pub images: Vec<String>,
    pub error: Option<String>,
}
