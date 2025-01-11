//! See:
//! - https://docs.aws.amazon.com/nova/latest/userguide/invoke.html
//! - https://docs.aws.amazon.com/nova/latest/userguide/complete-request-schema.html

use clap::Args;
// ----- JSON Considerations -----
// Had to do some serde field name changes in the types below to match the schema.
//
// https://serde.rs/field-attrs.html
// https://serde.rs/variant-attrs.html
// https://serde.rs/attr-skip-serializing.html
//
// https://stackoverflow.com/questions/59167416/how-can-i-deserialize-an-enum-when-the-case-doesnt-match
// https://stackoverflow.com/questions/53900612/how-do-i-avoid-generating-json-when-serializing-a-value-that-is-null-or-a-defaul
// --------------------------------
//
// Nova Lite can oly be accessed through cross-region inference
// https://docs.aws.amazon.com/bedrock/latest/userguide/cross-region-inference.html
//
use serde::{Deserialize, Serialize};

use crate::{BedrockSerde, DownloadLocation};

// ----------------------
// CLAP Stuff
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

    /// User prompt.
    ///
    /// The actual user prompt.
    user: String,
}

// ----------------------
// JSON Serde
// ----------------------

/// See:
/// - https://docs.aws.amazon.com/nova/latest/userguide/invoke.html
/// - https://docs.aws.amazon.com/nova/latest/userguide/complete-request-schema.html
#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
pub struct SystemPrompt {
    pub text: String,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Content {
    Text(String),
    Image(Image),
    Video(Video),
}

#[derive(Serialize, Deserialize)]
pub struct Image;

#[derive(Serialize, Deserialize)]
pub struct Video;

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

    // https://serde.rs/field-attrs.html
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
// Conversion
// -------------------
pub struct NovaBedrock(&'static str, Request);
impl From<NovaLiteArgs> for NovaBedrock {
    fn from(value: NovaLiteArgs) -> Self {
        let mut messages = vec![];
        messages.push(Message {
            role: Role::User,
            content: vec![Content::Text(value.user)],
        });

        if let Some(prefill) = value.assistant {
            messages.push(Message {
                role: Role::Assistant,
                content: vec![Content::Text(prefill)],
            });
        }

        let mut system = vec![];
        if let Some(prompt) = value.system {
            system.push(SystemPrompt { text: prompt });
        }

        // TODO allow this to be configured, maybe
        let inference_config: InferenceConfig = Default::default();

        let request = Request {
            system,
            messages,
            inference_config,
        };

        // nova lite requires invocation wth an inference profile id, instead of
        // its model-id: amazon.nova-lite-v1:0
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

    fn render_response(&self, body: String) -> (String, Vec<DownloadLocation>) {
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
fn rando() {
    let mut messages = vec![];
    let msg1 = Message {
        role: Role::User,
        content: vec![
            Content::Text("What a wonderful world".to_owned()),
            Content::Image(Image {
                foo1: "image1".to_owned(),
                foo2: "image2".to_owned(),
            }),
            Content::Video(Video {
                bar1: "video1".to_owned(),
                bar2: "video2".to_owned(),
            }),
        ],
    };
    messages.push(msg1);

    let request = Request {
        system: SystemPrompt {
            text: "hey there".to_owned(),
        },
        messages: messages,
        inference_config: InferenceConfig {
            max_new_tokens: None,
            temperature: None,
            top_p: Some(7.0),
            top_k: None,
            stop_sequences: vec![],
        },
    };

    let s = serde_json::to_string_pretty(&request).unwrap();
    println!("Request:\n{}\n------------\n", s);

    let role: Result<Role, serde_json::Error> = serde_json::from_str("\"assistant\"");
    println!("ROLE: {:?}", role);

    let role: Result<Role, serde_json::Error> = serde_json::from_str("\"user\"");
    println!("ROLE: {:?}", role);

    let cfg = InferenceConfig {
        max_new_tokens: None,
        temperature: None,
        top_p: None,
        top_k: None,
        stop_sequences: vec![],
    };
    let s = serde_json::to_string(&cfg).unwrap();
    println!("Inference Config\n{}", s);
}
