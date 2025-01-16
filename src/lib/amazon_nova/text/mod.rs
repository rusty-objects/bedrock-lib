use aws_sdk_bedrockruntime::operation::RequestId;
use json::InferenceConfig;
use log::debug;

use crate::file::{self, FileReference};
use crate::TraceId;

pub mod json;

pub async fn invoke_model(
    client: &aws_sdk_bedrockruntime::Client,
    model_id: String,
    inference_config: Option<InferenceConfig>,
    attachments: Vec<FileReference>,
    system_prompt: Option<String>,
    assistant_prefill: Option<String>,
    user_prompt: String,
) -> (TraceId, String) {
    // --------------
    // User content of the message.
    // This is required and must be the first content in the message list.
    //
    // User content may contain several elements, including multi-modal.
    // --------------
    let mut user_content = vec![];

    // add text
    user_content.push(json::Content::Text(user_prompt));

    // add media attachments
    for attachment in attachments {
        match (attachment.file_type, attachment.location) {
            (file::Type::Image, file::Location::Local) => {
                let base64 = file::read_base64(&attachment.path);
                user_content.push(json::Content::Image(json::Image {
                    format: attachment.extension.0,
                    source: json::ImageSource {
                        bytes: base64.unwrap(),
                    },
                }));
            }
            (file::Type::Video, file::Location::Local) => {
                let base64 = file::read_base64(&attachment.path);
                user_content.push(json::Content::Video(json::Video {
                    format: attachment.extension.0,
                    source: json::VideoSource::Bytes(base64.unwrap()),
                }));
            }
            (file::Type::Video, file::Location::S3) => {
                user_content.push(json::Content::Video(json::Video {
                    format: attachment.extension.0,
                    source: json::VideoSource::S3Location(json::S3Location {
                        uri: attachment.path,
                    }),
                }));
            }
            _ => panic!("Unsupported file type: {}", attachment.path),
        }
    }

    // add now-complete user_content to messages
    let mut messages = vec![json::Message {
        role: json::Role::User,
        content: user_content,
    }];

    // --------------
    // The assistant content (aka "prefill") of the message.
    // Optional and must occur last.
    //
    // https://www.walturn.com/insights/mastering-prompt-engineering-for-claude
    // --------------
    if let Some(prefill) = assistant_prefill {
        messages.push(json::Message {
            role: json::Role::Assistant,
            content: vec![json::Content::Text(prefill)],
        });
    }

    // ===============
    // The system prompt.  Optional.
    //
    // https://www.walturn.com/insights/mastering-prompt-engineering-for-claude
    // ===============
    let mut system = vec![];
    if let Some(text) = system_prompt {
        system.push(json::SystemPrompt { text });
    }

    let request = json::TextRequest {
        system,
        messages,
        inference_config: inference_config.unwrap_or_default(),
    };

    debug!("model-id: {}", model_id);
    debug!("{}", request.to_string());

    // ===============
    // Send request to Amazon Bedrock
    // https://docs.rs/aws-sdk-bedrockruntime/latest/aws_sdk_bedrockruntime/struct.Client.html#method.invoke_model
    // ===============
    let result = client
        .invoke_model()
        .content_type("application/json")
        .accept("application/json")
        .model_id(model_id.clone())
        .body(request.to_string().into_bytes().into())
        .send()
        .await;

    // Process the results, pretty printing the output
    if let Ok(value) = result {
        let body_ref = value.body.as_ref();
        let body = String::from_utf8(body_ref.to_owned()).unwrap();

        // printing the result will redact the contents of the body, so we print explicitly
        debug!("{:?}", value);
        debug!("{}", body);

        let rsp: json::Response = serde_json::from_str(body.as_str())
            .unwrap_or_else(|err| panic!("malformed json: err: {:?}, body:{}", err, body));
        let msg = rsp.output.message;

        assert_eq!(json::Role::Assistant, msg.role);

        if msg.content.len() != 1 {
            panic!("response content didn't have single element?\n{}", body);
        }

        let content = &msg.content[0];
        match content {
            json::Content::Text(val) => {
                let trace_id: TraceId =
                    TraceId(value.request_id().unwrap_or("UNKNOWN").to_string());
                return (trace_id, val.clone());
            }
            json::Content::Image(_) => {
                unimplemented!("{} doesn't support image output modality", model_id.clone())
            }
            json::Content::Video(_) => {
                unimplemented!("{} doesn't support video output modality", model_id)
            }
        }
    }
    panic!("bad response from bedrock:\n{:#?}", result);
}
