use clap::Parser;
use genlib::invoke_model::amazon::nova;

/// Invokes Amazon's Nova family of text models on Bedrock
///
/// Creative content models (Canvas and Reel) are not supported by this tool.
///
/// For more information on Amazon Nova, read the user guide:
///     https://docs.aws.amazon.com/nova/latest/userguide/
///
/// You must be opted into the model specified in you AWS account:
///     https://docs.aws.amazon.com/bedrock/latest/userguide/model-access.html
///
/// You must also have permission for bedrock:InvokeModel:
///     https://docs.aws.amazon.com/bedrock/latest/APIReference/API_runtime_InvokeModel.html
///
/// Example usage;
///     nova --image ~/black_dog.jpeg --image ~/white_dog.jpeg "What is the difference between these dogs?"
#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, verbatim_doc_comment)]
struct CliArgs {
    /// AWS profile override
    ///
    /// AWS region and credentials are selected in the following sequence:
    ///
    /// 1/ Explicit Override:
    ///     When this --profile option is specified, the named profile will be read from
    ///     ~/.aws/config and ~/.aws/credentials.
    ///
    /// 2/ Environment Variables, as described here:
    ///
    ///     https://docs.aws.amazon.com/cli/v1/userguide/cli-configure-envvars.html
    ///
    /// 3/ Default profile:
    ///     Uses the default profile from ~/.aws/config and ~/.aws/credentials.
    ///
    /// See the AWS docs for more information:
    ///
    ///     https://docs.aws.amazon.com/sdkref/latest/guide/file-format.html
    ///     https://docs.aws.amazon.com/sdk-for-rust/latest/dg/region.html
    ///     https://docs.aws.amazon.com/sdk-for-rust/latest/dg/credproviders.html
    #[clap(short = 'p', long, verbatim_doc_comment)]
    aws_profile: Option<String>,

    /// dumps raw input/output
    #[clap(short, long)]
    debug: bool,

    /// System prompt.
    ///
    /// Provides a system prompt for the model.
    ///
    /// See:
    ///     https://docs.aws.amazon.com/bedrock/latest/userguide/prompt-management-create.
    ///     https://docs.aws.amazon.com/nova/latest/userguide/invoke.html#utilizing-system-prompt
    ///     https://www.regie.ai/blog/user-prompts-vs-system-prompts
    ///     https://docs.aws.amazon.com/bedrock/latest/userguide/prompt-management-create
    #[clap(short, long, verbatim_doc_comment)]
    system: Option<String>,

    /// The model to use.  Default: us.amazon.nova-lite-v1:0
    ///
    /// Amazon Bedrock requires using an inference profile for Amazon Nova models
    /// rather than calling InvokeModel directly on the model id.  Example valid
    /// values for this field are:
    ///
    /// - Micro:
    ///     - model-id: amazon.nova-micro-v1:0
    ///     - inference-profile-id: us.amazon.nova-micro-v1:0
    /// - Lite (default):
    ///     - model-id: amazon.nova-lite-v1:0
    ///     - inference-profile-id: us.amazon.nova-lite-v1:0
    /// - Pro:
    ///     - model-id: amazon.nova-pro-v1:0
    ///     - inference-profile-id: us.amazon.nova-pro-v1:0
    ///
    /// Not all models support all modalities (e.g. micro doesn't accept image/video input).
    ///
    /// For more information, visit:
    ///
    ///     https://docs.aws.amazon.com/nova/latest/userguide/
    ///     https://docs.aws.amazon.com/bedrock/latest/userguide/cross-region-inference.html
    #[clap(
        short,
        long,
        default_value = "us.amazon.nova-lite-v1:0",
        verbatim_doc_comment
    )]
    model: String,

    /// Prefilled assistant response.
    ///
    /// If provided, then when this model is invoked this prompt will be sent to the model for it to use to start off its answer.
    #[clap(short, long)]
    assistant: Option<String>,

    /// Paths for image content to send
    ///
    /// This tool won't validate the files are supported.  Not all models support
    /// all modalities.
    ///
    /// See:
    ///     https://docs.aws.amazon.com/nova/latest/userguide/modalities.html
    #[clap(short, long, verbatim_doc_comment)]
    image: Vec<String>,

    /// Paths for video content to send
    ///
    /// This tool won't validate the files are supported.  Not all models support
    /// all modalities.
    ///
    /// See:
    ///     https://docs.aws.amazon.com/nova/latest/userguide/modalities.html
    #[clap(short, long, verbatim_doc_comment)]
    video: Vec<String>,

    /// S3 Uri for video content to send
    ///
    /// This tool won't validate the files are supported.  Not all models support
    /// all modalities.
    ///
    /// See:
    ///     https://docs.aws.amazon.com/nova/latest/userguide/modalities.html
    ///
    /// Note: Amazon Nova supports cross-account S3 access but this tool does not.
    /// That would require modifying the tool to accept the bucket owner account id
    /// in the model invocation.
    #[clap(short, long, verbatim_doc_comment)]
    uri_video: Vec<String>,

    /// User prompt.
    ///
    /// The actual user prompt.
    prompt: String,
}

// #[async_std::main]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = CliArgs::parse();

    // Wire up SdkConfig:
    // https://docs.rs/aws-config/latest/aws_config/
    // https://docs.aws.amazon.com/cli/v1/userguide/cli-configure-files.html
    // https://docs.aws.amazon.com/sdk-for-rust/latest/dg/configure.html
    // https://docs.aws.amazon.com/sdkref/latest/guide/file-format.html
    // https://docs.aws.amazon.com/sdk-for-rust/latest/dg/credproviders.html
    // https://docs.rs/aws-config/latest/aws_config/profile/credentials/struct.ProfileFileCredentialsProvider.html
    // https://docs.rs/aws-config/latest/aws_config/profile/struct.ProfileFileRegionProvider.html
    let config = if let Some(profile) = cli.aws_profile.clone() {
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
    let client = aws_sdk_bedrockruntime::Client::new(&config);

    let request: nova::Request = cli.clone().into();

    if cli.debug {
        println!(">>> request");
        println!("id: {}", cli.model);
        println!("{}", request.to_string());
    }

    // Send InvokeModel to Amazon Bedrock
    //
    // https://docs.rs/aws-sdk-bedrockruntime/latest/aws_sdk_bedrockruntime/struct.Client.html#method.invoke_model
    let result = client
        .invoke_model()
        .content_type("application/json")
        .accept("application/json")
        .model_id(cli.model)
        .body(request.to_string().into_bytes().into())
        .send()
        .await;

    // Process the results, pretty printing the output
    match result {
        Ok(result) => {
            let body = result.clone().body;
            let body_bytes = body.as_ref();
            let body_string = String::from_utf8(body_bytes.to_owned()).unwrap();

            if cli.debug {
                println!("\n<<< response\n{:#?}", result);

                // printing the result will redact the contents of the body, so we print explicitly
                println!("{}\n", body_string);
            }

            println!("{}", parse_response(body_string));
        }
        Err(result) => println!("\nerror:\n{:#?}", result),
    }

    Ok(())
}

impl From<CliArgs> for nova::Request {
    fn from(value: CliArgs) -> Self {
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
        user_content.push(nova::Content::Text(value.prompt));

        // add inline images
        for image in value.image {
            let format = genlib::file::get_extension_from_filename(&image);
            let base64 = genlib::file::read_base64(&image);
            user_content.push(nova::Content::Image(nova::Image {
                format,
                source: nova::ImageSource { bytes: base64 },
            }));
        }

        // add inline videos
        for video in value.video {
            let format = genlib::file::get_extension_from_filename(&video);
            let base64 = genlib::file::read_base64(&video);
            user_content.push(nova::Content::Video(nova::Video {
                format,
                source: nova::VideoSource::Bytes(base64),
            }));
        }

        // add s3 videos
        for uri in value.uri_video {
            let format = genlib::file::get_extension_from_filename(&uri);
            user_content.push(nova::Content::Video(nova::Video {
                format,
                source: nova::VideoSource::S3Location(nova::S3Location { uri }),
            }));
        }

        // add now-complete user_content to messages
        messages.push(nova::Message {
            role: nova::Role::User,
            content: user_content,
        });

        // --------------
        // The assistant content (aka "prefill") of the message.
        // Optional and must occur last.
        //
        // https://www.walturn.com/insights/mastering-prompt-engineering-for-claude
        // --------------
        if let Some(prefill) = value.assistant {
            messages.push(nova::Message {
                role: nova::Role::Assistant,
                content: vec![nova::Content::Text(prefill)],
            });
        }

        // ===============
        // The system prompt.  Optional.
        //
        // https://www.walturn.com/insights/mastering-prompt-engineering-for-claude
        // ===============
        let mut system = vec![];
        if let Some(prompt) = value.system {
            system.push(nova::SystemPrompt { text: prompt });
        }

        // ===============
        // Inference configuration
        // TODO allow this to be configured, maybe
        // ===============
        let inference_config: nova::InferenceConfig = Default::default();

        nova::Request {
            system,
            messages,
            inference_config,
        }
    }
}

fn parse_response(body: String) -> String {
    let rsp: nova::Response = serde_json::from_str(body.as_str())
        .unwrap_or_else(|err| panic!("JSON was not well-formatted: err: {:?}, body:{}", err, body));
    let msg = rsp.output.message;

    assert_eq!(nova::Role::Assistant, msg.role);

    let mut s = None;
    for content in msg.content {
        match content {
            nova::Content::Text(val) => match s {
                None => s = Some(val),
                Some(_) => panic!("content with multiple text elements? {}", body),
            },
            nova::Content::Image(_) => unimplemented!("nova doesn't support image output modality"),
            nova::Content::Video(_) => unimplemented!("nova doesn't support video output modality"),
        }
    }

    s.unwrap_or_default()
}

#[test]
fn conversion() {
    let prompt = "x".to_owned();
    let args = CliArgs {
        aws_profile: None,
        debug: false,
        model: Default::default(),
        system: None,
        assistant: None,
        image: vec![],
        video: vec![],
        uri_video: vec![],
        prompt: prompt.clone(),
    };

    let request = Into::<nova::Request>::into(args);

    assert_eq!(1, request.messages.len());
    assert_eq!(nova::Role::User, request.messages[0].role);
    assert_eq!(1, request.messages[0].content.len());
    match &(request.messages[0].content[0]) {
        nova::Content::Text(c) => assert_eq!(c, &prompt),
        _ => panic!("bad content"),
    }
}
