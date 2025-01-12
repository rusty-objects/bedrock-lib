//! Specific implementation of InvokeModel request/response structs Amazon Nova models
//!
//! The rust structs here are set up so that serde generates compatible json according
//! to the published request schema:
//!
//! - https://docs.aws.amazon.com/nova/latest/userguide/complete-request-schema.html
//!
//! Note: I cannot find a published response schema, so the structs here are based on
//! observed responses

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
pub struct Request {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub system: Vec<SystemPrompt>,

    /// First message in the list MUST have a user role, and then they alternate from
    /// there (if calling Converse).  
    pub messages: Vec<Message>,

    #[serde(rename = "inferenceConfig")]
    #[serde(skip_serializing_if = "InferenceConfig::is_empty")]
    pub inference_config: InferenceConfig,
    // toolConfig: ToolConfig, // TODO
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemPrompt {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub role: Role,
    pub content: Vec<Content>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Content {
    Text(String),
    Image(Image),
    Video(Video),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    pub format: String,
    pub source: ImageSource,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageSource {
    pub bytes: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Video {
    pub format: String,
    pub source: VideoSource,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum VideoSource {
    #[serde(rename = "s3Location")]
    S3Location(S3Location),
    #[serde(rename = "bytes")]
    Bytes(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct S3Location {
    pub uri: String,
    // TODO if you ever want to support cross account requests, need to add this
    // #[serde(rename = "bucketOwner")]
    // pub bucket_owner: String,
}

// TODO make this configurable via CLI args
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct InferenceConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_new_tokens: Option<u16>, // greater than 0, equal or less than 5k (default: dynamic*)

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>, // greater then 0 and less than 1.0 (default: 0.7)

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>, // greater than 0, equal or less than 1.0 (default: 0.9)

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>, // 0 or greater (default: 50)

    #[serde(rename = "stopSequences")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stop_sequences: Vec<String>,
}
impl InferenceConfig {
    // have serde skip including inference config altogether if no values are present
    pub fn is_empty(&self) -> bool {
        self.max_new_tokens.is_none()
            && self.temperature.is_none()
            && self.top_p.is_none()
            && self.top_k.is_none()
            && self.stop_sequences.is_empty()
    }
}

impl ToString for Request {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

/// Can't find any documented response schema, so this is reverse engieered from a sample:
///
/// ```text
/// {
///   "output": {
///       "message": {
///           "content": [
///               {
///                   "text": "Hello!"
///               }
///           ],
///           "role": "assistant"
///       }
///   },
///   "stopReason": "end_turn",
///   "usage": {
///       "inputTokens": 4,
///       "outputTokens": 35,
///       "totalTokens": 39
///   }
/// }
/// ```
///
/// See:
/// - https://docs.aws.amazon.com/nova/latest/userguide/invoke.html
/// - https://docs.aws.amazon.com/nova/latest/userguide/complete-request-schema.html
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub output: Output,
    pub stop_reason: String,
    pub usage: Usage,
}

#[derive(Serialize, Deserialize)]
pub struct Output {
    pub message: Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

#[test]
fn video_encoding() {
    let video1 = Video {
        format: "abc".to_owned(),
        source: VideoSource::Bytes("123".to_owned()),
    };

    println!(
        "bytes\n{}\n",
        serde_json::to_string_pretty(&video1).unwrap()
    );

    let video2 = Video {
        format: "abc".to_owned(),
        source: VideoSource::S3Location(S3Location {
            uri: "s3uri".to_owned(),
        }),
    };

    println!("s3\n{}", serde_json::to_string_pretty(&video2).unwrap());
}
