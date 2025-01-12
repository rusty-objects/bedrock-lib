//! Specific implementation of invoke for Amazon Nova models
//!
//! See:
//! - https://docs.aws.amazon.com/nova/latest/userguide/invoke.html
//! - https://docs.aws.amazon.com/nova/latest/userguide/complete-request-schema.html
//!
//! For requests, this module takes user input (via rust structs encoded by clap), and converts
//! them to model-specific json inference configuration.  For multi-media inputs, this module reads
//! and encodes the data from disk appropriately.
//!
//! For responses, this module reads the json inference response from the model, writes any media to disk
//! and redners the output.
//!
//! The field names of these structs match the Amazon Nova documentation.
//!
//! Note: this module doens't interact with Bedrock and isn't aware of bedrock APIs.  It's encoding the body
//! portion of bedrock requests only.

use clap::Args;

// Had to do some serde field name changes in the types below to match the schema.
//
// https://serde.rs/field-attrs.html
// https://serde.rs/variant-attrs.html
// https://serde.rs/attr-skip-serializing.html
//
// https://stackoverflow.com/questions/59167416/how-can-i-deserialize-an-enum-when-the-case-doesnt-match
// https://stackoverflow.com/questions/53900612/how-do-i-avoid-generating-json-when-serializing-a-value-that-is-null-or-a-defaul
use serde::{Deserialize, Serialize};

use crate::{BedrockSerde, DownloadLocation};

// ----------------------
// CLAP Types
// ----------------------

/// Amazon Nova Lite v1:0
///
/// Model will be invoked via the inference profile, which allows bedrock to steer
/// requests to regions with available capacity.
///
/// - model-id: amazon.nova-lite-v1:0
/// - inference-profile-id: us.amazon.nova-lite-v1:0
///
/// For more details, refer to the nova user guide: https://docs.aws.amazon.com/nova/latest/userguide/
#[derive(Args, Debug)]
pub struct NovaLiteArgs {
    /// System prompt.
    ///
    /// Provides a system prompt for the model.
    ///
    /// See: https://docs.aws.amazon.com/bedrock/latest/userguide/prompt-management-create.
    #[clap(short, long)]
    system: Option<String>,

    /// Prefilled assistant response.
    ///
    /// If provided, then when this model is invoked this prompt will be sent to the model for itt o use to start off its answer.
    #[clap(short, long)]
    assistant: Option<String>,

    /// Paths for image content to send
    ///
    /// This tool won't validate the files are supported.  Not all models support
    /// all modalities.
    ///
    /// See: https://docs.aws.amazon.com/nova/latest/userguide/modalities.html
    #[clap(short, long)]
    image: Vec<String>,

    /// Paths for video content to send
    ///
    /// This tool won't validate the files are supported.  Not all models support
    /// all modalities.
    ///
    /// See: https://docs.aws.amazon.com/nova/latest/userguide/modalities.html
    #[clap(short, long)]
    video: Vec<String>,

    /// S3 Uri for video content to send to the model.  
    ///
    /// This tool won't validate the files are supported.  Not all models support
    /// all modalities.
    ///
    /// See: https://docs.aws.amazon.com/nova/latest/userguide/modalities.html
    ///
    /// Note: Amazon Nova supports cross-account S3 access but this tool does not.
    /// That would require modifying the tool to accept the bucket owner account id
    /// in the model invocation.
    #[clap(short, long)]
    uri_video: Vec<String>,

    /// User prompt.
    ///
    /// The actual user prompt.
    user: String,
}

// ----------------------
// JSON Serde Types
// ----------------------

/// See:
/// - https://docs.aws.amazon.com/nova/latest/userguide/invoke.html
/// - https://docs.aws.amazon.com/nova/latest/userguide/complete-request-schema.html
#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub system: Vec<SystemPrompt>,

    /// First message in the list MUST be a user role, and then they alternate from
    /// there (if calling Converse).  
    pub messages: Vec<Message>, // first must be a user message

    #[serde(rename = "inferenceConfig")]
    #[serde(skip_serializing_if = "InferenceConfig::is_empty")]
    pub inference_config: InferenceConfig,
    // toolConfig: ToolConfig, // TODO
}

/// See:
/// - https://docs.aws.amazon.com/nova/latest/userguide/invoke.html#utilizing-system-prompt
/// - https://www.regie.ai/blog/user-prompts-vs-system-prompts
/// - https://docs.aws.amazon.com/bedrock/latest/userguide/prompt-management-create.
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
    format: String,
    source: ImageSource,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImageSource {
    // TODO unclear how this needs to look for the "binary array" required
    // for converse, which doesn't take the base64 that InvokeModel takes
    bytes: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Video {
    format: String,
    source: VideoSource,
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
    uri: String,
    // #[serde(rename = "bucketOwner")]
    // bucket_owner: String, // optional, for cross-account requests
}

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
    pub fn is_empty(&self) -> bool {
        self.max_new_tokens.is_none()
            && self.temperature.is_none()
            && self.top_p.is_none()
            && self.top_k.is_none()
            && self.stop_sequences.is_empty()
    }
}

/// Can't find any docmentation on the response schema, so this is reverse engieered from
/// a response:
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
    stop_reason: String,
    usage: Usage,
}

#[derive(Serialize, Deserialize)]
pub struct Output {
    message: Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    input_tokens: u32,
    output_tokens: u32,
    total_tokens: u32,
}

// -------------------
// CLAP Struct -> JSON Struct conversion
// Bedrock Response Printer
// -------------------
pub struct NovaBedrock(&'static str, Request);
#[cfg(test)]
impl NovaBedrock {
    pub fn unwrap(self) -> (&'static str, Request) {
        (self.0, self.1)
    }
}

impl From<NovaLiteArgs> for NovaBedrock {
    fn from(value: NovaLiteArgs) -> Self {
        // ==============
        // The messages to be sent
        // ==============
        let mut messages = vec![];

        // --------------
        // User content of the message.
        // This is required and must be the first content in the message list.
        //
        // User content may contain several elements, including multi-modal.
        // --------------
        let mut user_content = vec![];

        // add text
        user_content.push(Content::Text(value.user));

        // add inline images
        for image in value.image {
            let format = crate::file::get_extension_from_filename(&image);
            let base64 = crate::file::read_base64(&image);
            user_content.push(Content::Image(Image {
                format,
                source: ImageSource { bytes: base64 },
            }));
        }

        // add inline videos
        for video in value.video {
            let format = crate::file::get_extension_from_filename(&video);
            let base64 = crate::file::read_base64(&video);
            user_content.push(Content::Video(Video {
                format,
                source: VideoSource::Bytes(base64),
            }));
        }

        // add s3 videos
        for uri in value.uri_video {
            let format = crate::file::get_extension_from_filename(&uri);
            user_content.push(Content::Video(Video {
                format,
                source: VideoSource::S3Location(S3Location { uri }),
            }));
        }

        // add now-complete user_content to messages
        messages.push(Message {
            role: Role::User,
            content: user_content,
        });

        // --------------
        // The assistant content (aka "prefill") of the message.
        // Optional and must occur last.
        //
        // https://www.walturn.com/insights/mastering-prompt-engineering-for-claude
        // --------------
        if let Some(prefill) = value.assistant {
            messages.push(Message {
                role: Role::Assistant,
                content: vec![Content::Text(prefill)],
            });
        }

        // ===============
        // The system prompt.  Optional.
        //
        // https://www.walturn.com/insights/mastering-prompt-engineering-for-claude
        // ===============
        let mut system = vec![];
        if let Some(prompt) = value.system {
            system.push(SystemPrompt { text: prompt });
        }

        // ===============
        // Inference configuration
        // TODO allow this to be configured, maybe
        // ===============
        let inference_config: InferenceConfig = Default::default();

        let request = Request {
            system,
            messages,
            inference_config,
        };

        // Nova Lite can only be accessed through cross-region inference
        // https://docs.aws.amazon.com/bedrock/latest/userguide/cross-region-inference.html
        //
        // This requires invocation wth an inference profile id, instead of model-id (amazon.nova-lite-v1:0)
        let model_id = "us.amazon.nova-lite-v1:0";

        NovaBedrock(model_id, request)
    }
}

impl BedrockSerde for NovaBedrock {
    fn model_id(&self) -> &str {
        self.0
    }

    fn body(&self) -> String {
        serde_json::to_string(&self.1).unwrap()
    }

    fn render_response(
        &self,
        body: String,
        _base_write_path: String,
    ) -> (String, Vec<DownloadLocation>) {
        let rsp: Response = serde_json::from_str(body.as_str()).unwrap_or_else(|err| {
            panic!("JSON was not well-formatted: err: {:?}, body:{}", err, body)
        });
        let msg = rsp.output.message;

        assert_eq!(Role::Assistant, msg.role);

        // TODO: this will change with multi-modal responses
        let mut s = None;
        let locations = vec![];
        for content in msg.content {
            match content {
                Content::Text(val) => match s {
                    None => s = Some(val),
                    Some(_) => panic!("Response had multiple texts? {}", body),
                },
                Content::Image(_val) => unimplemented!("image"),
                Content::Video(_val) => unimplemented!("video"),
            }
        }

        (s.unwrap_or_default(), locations)
    }
}

#[test]
fn conversion() {
    let user = "x".to_owned();
    let args = NovaLiteArgs {
        system: None,
        assistant: None,
        image: vec![],
        video: vec![],
        uri_video: vec![],
        user: user.clone(),
    };

    let (_, request) = Into::<NovaBedrock>::into(args).unwrap();

    assert_eq!(1, request.messages.len());
    assert_eq!(Role::User, request.messages[0].role);
    assert_eq!(1, request.messages[0].content.len());
    match &(request.messages[0].content[0]) {
        Content::Text(c) => assert_eq!(c, &user),
        _ => panic!("bad content"),
    }
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
