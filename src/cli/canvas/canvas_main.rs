use clap::Parser;
use rusty_bedrock_lib::nova::canvas;

/// Invokes Amazon's Canvas model on Bedrock
///
/// model-id: amazon.nova-canvas-v1:0
///
/// You must be opted into the model specified in you AWS account have have
/// `bedrock:InvokeModel` permissions:
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
struct CanvasCliArgs {
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
    #[clap(long, verbatim_doc_comment)]
    aws_profile: Option<String>,

    /// prints request/response detail
    #[clap(short, long)]
    verbose: bool,

    /// Output directory
    #[clap(short, long, default_value = ".")]
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

#[tokio::main]
async fn main() {
    let cli: CanvasCliArgs = CanvasCliArgs::parse();

    let verbosity = if cli.verbose { 3 } else { 2 };
    stderrlog::new().verbosity(verbosity).init().unwrap();

    // https://docs.rs/aws-sdk-bedrockruntime/latest/aws_sdk_bedrockruntime/
    let client = rusty_bedrock_lib::new_runtime_client(cli.aws_profile).await;

    let (trace_id, images) = canvas::text_to_image(&client, cli.prompt, cli.negative).await;

    let outdir = cli.output.trim_end_matches('/').to_string();
    for (idx, image) in images.into_iter().enumerate() {
        if idx == 0 {
            println!("Writing:")
        }
        let path = format!("{}/{}-{}.png", outdir, trace_id, idx);
        rusty_bedrock_lib::file::write_base64(path.as_str(), image.as_ref().to_string());
        println!("{}", path);
    }
}
