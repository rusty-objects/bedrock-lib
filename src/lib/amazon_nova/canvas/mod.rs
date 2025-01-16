use aws_sdk_bedrockruntime::operation::RequestId;
use json::{CanvasRequest, CanvasResponse, TextToImageParams};
use log::debug;

use crate::{file::Base64Encoding, TraceId};

pub mod json;

static MODEL_ID: &str = "amazon.nova-canvas-v1:0";

pub async fn text_to_image(
    client: &aws_sdk_bedrockruntime::Client,
    prompt: String,
    negative_prompt: Option<String>,
) -> (TraceId, Vec<Base64Encoding>) {
    let params = TextToImageParams {
        text: prompt,
        negative_text: negative_prompt.unwrap_or_default(),
    };

    let request = CanvasRequest {
        task_type: "TEXT_IMAGE".to_owned(),
        text_to_image_params: params,
        image_generation_config: None,
    };

    debug!("model-id: {}", MODEL_ID);
    debug!("{}", request.to_string());

    // https://docs.rs/aws-sdk-bedrockruntime/latest/aws_sdk_bedrockruntime/struct.Client.html#method.invoke_model
    let result = client
        .invoke_model()
        .content_type("application/json")
        .accept("application/json")
        .model_id(MODEL_ID)
        .body(request.to_string().into_bytes().into())
        .send()
        .await;

    // Process the results, pretty printing the output
    match result {
        Ok(result) => {
            let body_vec = result.body.as_ref().to_owned();
            let body = String::from_utf8(body_vec).unwrap();

            debug!("{:?}", result);
            debug!("{} ... {}", &body[0..50], &body[body.len() - 50..]);

            let rsp: CanvasResponse = serde_json::from_str(&body)
                .unwrap_or_else(|err| panic!("malformed json: \nbody: {:?} \nerr:{}", body, err));

            if let Some(error) = rsp.error {
                panic!("InvokeModelOutput.error:\n{}", error);
            }

            let trace_id: TraceId = TraceId(result.request_id().unwrap_or("UNKNOWN").to_string());

            (
                trace_id,
                rsp.images
                    .into_iter()
                    .map(|s| Base64Encoding::new(s))
                    .collect(),
            )
        }
        Err(result) => panic!("InvokeModelError:\n{:#?}", result),
    }
}
