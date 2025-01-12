//! Specific implementation of InvokeModel request/response structs Amazon Canvas model
//!
//! The rust structs here are set up so that serde generates compatible json according
//! to the published request and response schemas:
//!
//! https://docs.aws.amazon.com/nova/latest/userguide/image-gen-req-resp-structure.html
//!

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
pub struct Request {
    pub task_type: String,
    pub text_to_image_params: TextToImageParams,

    #[serde(skip)]
    pub image_generation_config: ImageGenerationConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TextToImageParams {
    pub text: String,

    #[serde(rename = "negativeText", skip_serializing_if = "String::is_empty")]
    pub negative_text: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ImageGenerationConfig;

impl ToString for Request {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub images: Vec<String>,
    pub error: Option<String>,
}
