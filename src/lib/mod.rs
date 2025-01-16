pub mod amazon_nova;
pub mod converse;
pub mod file;

use std::{collections::HashMap, fmt::Display};

pub use amazon_nova as nova;
use aws_sdk_bedrock::types::InferenceType;

pub struct TraceId(String);
impl AsRef<str> for TraceId {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
impl Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

pub async fn new_runtime_client(aws_profile: Option<String>) -> aws_sdk_bedrockruntime::Client {
    // Wire up SdkConfig:
    // https://docs.rs/aws-config/latest/aws_config/
    // https://docs.aws.amazon.com/cli/v1/userguide/cli-configure-files.html
    // https://docs.aws.amazon.com/sdk-for-rust/latest/dg/configure.html
    // https://docs.aws.amazon.com/sdkref/latest/guide/file-format.html
    // https://docs.aws.amazon.com/sdk-for-rust/latest/dg/credproviders.html
    // https://docs.rs/aws-config/latest/aws_config/profile/credentials/struct.ProfileFileCredentialsProvider.html
    // https://docs.rs/aws-config/latest/aws_config/profile/struct.ProfileFileRegionProvider.html
    let config = if let Some(profile) = aws_profile.clone() {
        aws_config::from_env()
            .credentials_provider(
                aws_config::profile::ProfileFileCredentialsProvider::builder()
                    .profile_name(profile.clone())
                    .build(),
            )
            .region(
                aws_config::profile::ProfileFileRegionProvider::builder()
                    .profile_name(profile)
                    .build(),
            )
            .load()
            .await
    } else {
        aws_config::load_from_env().await
    };

    // https://docs.rs/aws-sdk-bedrockruntime/latest/aws_sdk_bedrockruntime/
    aws_sdk_bedrockruntime::Client::new(&config)
}

pub async fn new_controlplane_client(aws_profile: Option<String>) -> aws_sdk_bedrock::Client {
    // Wire up SdkConfig:
    // https://docs.rs/aws-config/latest/aws_config/
    // https://docs.aws.amazon.com/cli/v1/userguide/cli-configure-files.html
    // https://docs.aws.amazon.com/sdk-for-rust/latest/dg/configure.html
    // https://docs.aws.amazon.com/sdkref/latest/guide/file-format.html
    // https://docs.aws.amazon.com/sdk-for-rust/latest/dg/credproviders.html
    // https://docs.rs/aws-config/latest/aws_config/profile/credentials/struct.ProfileFileCredentialsProvider.html
    // https://docs.rs/aws-config/latest/aws_config/profile/struct.ProfileFileRegionProvider.html
    let config = if let Some(profile) = aws_profile.clone() {
        aws_config::from_env()
            .credentials_provider(
                aws_config::profile::ProfileFileCredentialsProvider::builder()
                    .profile_name(profile.clone())
                    .build(),
            )
            .region(
                aws_config::profile::ProfileFileRegionProvider::builder()
                    .profile_name(profile)
                    .build(),
            )
            .load()
            .await
    } else {
        aws_config::load_from_env().await
    };

    // https://docs.rs/aws-sdk-bedrock/latest/aws_sdk_bedrock/
    aws_sdk_bedrock::Client::new(&config)
}

/// Lists OnDemand models
pub async fn list_models(
    client: &aws_sdk_bedrock::Client,
    by_provider: Option<String>,
) -> Vec<ModelDetails> {
    let models = client
        .list_foundation_models()
        .by_inference_type(InferenceType::OnDemand)
        .set_by_provider(by_provider)
        .send()
        .await
        .unwrap()
        .model_summaries
        .unwrap();

    let profiles = client
        .list_inference_profiles()
        .send()
        .await
        .unwrap()
        .inference_profile_summaries
        .unwrap();

    let mut model_map = HashMap::new();
    for model in models {
        let model_id = model.model_id().to_owned();
        let name = model.model_name().unwrap().to_owned();
        let provider = model.provider_name().unwrap().to_owned();
        let arn = model.model_arn().to_owned();
        let input = model
            .input_modalities()
            .iter()
            .map(|m| m.to_string())
            .collect();
        let output = model
            .output_modalities()
            .iter()
            .map(|m| m.to_string())
            .collect();
        let details = ModelDetails {
            provider,
            name,
            model_id,
            input,
            output,
            inference_profiles: Vec::new(),
        };
        model_map.insert(arn, details);
    }

    for profile in profiles {
        let profile_id = profile.inference_profile_id().to_owned();
        for model in profile.models() {
            if let Some(model_details) = model_map
                .get_mut(model.model_arn().unwrap()) { model_details.inference_profiles.push(profile_id.clone()) }
        }
    }

    let mut vec = model_map
        .values().cloned()
        .collect::<Vec<_>>();
    vec.sort_by_key(|a| format!("{}{}", a.provider, a.name).to_string());
    vec
}

#[derive(Debug, Clone)]
pub struct ModelDetails {
    pub provider: String,
    pub name: String,
    pub model_id: String,
    pub input: Vec<String>,
    pub output: Vec<String>,
    pub inference_profiles: Vec<String>,
}
impl Display for ModelDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} | {} | model-id: {} | profile-ids: {} | inputs: {} | outputs: {}",
            self.provider,
            self.name,
            self.model_id,
            self.inference_profiles.join(", "),
            self.input.join(", "),
            self.output.join(", ")
        )
    }
}
