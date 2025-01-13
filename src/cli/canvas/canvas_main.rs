use aws_sdk_bedrockruntime::operation::RequestId;
use clap::Parser;
use genlib::invoke_model::amazon::canvas::{self, ImageGenerationConfig};

/// Invokes Amazon's Canvas model on Bedrock
///
/// model-id: amazon.nova-canvas-v1:0
///
/// You must be opted into the model specified in you AWS account have have
/// bedrock:InvokeModel permissions:
///     https://docs.aws.amazon.com/bedrock/latest/userguide/model-access.html
///     https://docs.aws.amazon.com/bedrock/latest/APIReference/API_runtime_InvokeModel.html
///
/// === Example usage ===
///
///     canvas --negative "birds, ducks" "Picture of a lake with wildlife, photorealistic"
///
/// For more information on Amazon Nova, read the user guide:
///     https://docs.aws.amazon.com/nova/latest/userguide/
///
/// === Future work ===
/// Will eventually use sub-commands for Canvas's other features like image editting:
///     https://docs.aws.amazon.com/nova/latest/userguide/image-generation.html
#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, verbatim_doc_comment)]
struct CliArgs {
    /// AWS profile override
    ///
    /// AWS region and credentials are selected in the following sequence:
    ///
    /// 1/ Explicit Override:
    /// When this --profile option is specified, the named profile will be read from
    ///     ~/.aws/config and ~/.aws/credentials.
    ///
    /// 2/ Environment Variables, as described here:
    ///     https://docs.aws.amazon.com/cli/v1/userguide/cli-configure-envvars.html
    ///
    /// 3/ Default profile:
    /// Uses the default profile from ~/.aws/config and ~/.aws/credentials.
    ///
    /// See the AWS docs for more information:
    ///   https://docs.aws.amazon.com/sdkref/latest/guide/file-format.html
    ///   https://docs.aws.amazon.com/sdk-for-rust/latest/dg/region.html
    ///   https://docs.aws.amazon.com/sdk-for-rust/latest/dg/credproviders.html
    #[clap(short = 'p', long, verbatim_doc_comment)]
    aws_profile: Option<String>,

    /// dumps raw input/output
    #[clap(short, long)]
    debug: bool,

    /// dumps input and output with the body redacted
    #[clap(short, long)]
    verbose: bool,

    /// Output directory
    #[clap(short, long, default_value = "./")]
    output: String,

    /// Negative prompt
    ///
    /// If provided, instructs Canvas what not to include.  Avoid negation words
    /// like "no" and "without"
    #[clap(short, long)]
    negative: Option<String>,

    /// User prompt.
    ///
    /// Canvas isn't conversational.  Try to structure the prompt to be more like an image
    /// caption.  Avoid negation words ("no", "without"), as that will have the opposite effect.
    /// Instead, provide a negative prompt for exclusions.
    prompt: String,
}

// #[async_std::main]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli: CliArgs = CliArgs::parse();

    // Wire up SdkConfig.  Various reading on the subject:
    //
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

    let request: canvas::Request = cli.clone().into();

    let model_id = "amazon.nova-canvas-v1:0";

    if cli.debug || cli.verbose {
        println!(">>> request");
        println!("id: {}", model_id);
        println!("{}", request.to_string());
    }

    // Send InvokeModel to Amazon Bedrock
    //
    // https://docs.rs/aws-sdk-bedrockruntime/latest/aws_sdk_bedrockruntime/struct.Client.html#method.invoke_model
    let result = client
        .invoke_model()
        .content_type("application/json")
        .accept("application/json")
        .model_id(model_id)
        .body(request.to_string().into_bytes().into())
        .send()
        .await;

    // Process the results, pretty printing the output
    match result {
        Ok(result) => {
            let body = result.clone().body;
            let body_bytes = body.as_ref();
            let body_string = String::from_utf8(body_bytes.to_owned()).unwrap();

            if cli.debug || cli.verbose {
                println!("\n<<< response\n{:#?}", result);
            }

            if cli.verbose {
                let len = body_bytes.len();
                println!(
                    "len: {}\n{} ... {}\n",
                    len,
                    body_string[0..50].to_owned(),
                    body_string[len - 50..].to_owned()
                );
                println!("run with --debug for more");
            } else if cli.debug {
                println!("{}\n", body_string);
            }

            let file_prefix = result.request_id().unwrap_or("out");
            let paths = parse_response(body_string, format!("{}/{}-", cli.output, file_prefix));
            if !paths.is_empty() {
                println!("Writing:")
            }
            for path in paths {
                println!("{}", path);
            }
        }
        Err(result) => println!("\nerror:\n{:#?}", result),
    }

    Ok(())
}

impl From<CliArgs> for canvas::Request {
    fn from(value: CliArgs) -> Self {
        let params = canvas::TextToImageParams {
            text: value.prompt,
            negative_text: value.negative.unwrap_or_default(),
        };

        canvas::Request {
            task_type: "TEXT_IMAGE".to_owned(),
            text_to_image_params: params,
            image_generation_config: ImageGenerationConfig,
        }
    }
}

fn parse_response(body: String, base_path: String) -> Vec<String> {
    let rsp: canvas::Response = serde_json::from_str(body.as_str())
        .unwrap_or_else(|err| panic!("JSON was not well-formatted: err: {:?}, body:{}", err, body));

    if let Some(error) = rsp.error {
        println!("response.error: {}", error);
    }

    let mut files = vec![];
    for (idx, image) in rsp.images.into_iter().enumerate() {
        let path = format!("{}{}.png", base_path, idx);
        genlib::file::write_base64(path.as_str(), image);
        files.push(path);
    }

    files
}

#[test]
fn conversion() {
    let prompt = "x".to_owned();
    let args = CliArgs {
        aws_profile: None,
        debug: false,
        verbose: false,
        output: Default::default(),
        negative: Default::default(),
        prompt: prompt.clone(),
    };

    let request = Into::<canvas::Request>::into(args);

    assert_eq!(request.text_to_image_params.text, prompt);
}
