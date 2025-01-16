use clap::Parser;
use rusty_bedrock_lib::{file::FileReference, nova};

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
    #[clap(long, verbatim_doc_comment)]
    aws_profile: Option<String>,

    /// prints request/response detail
    #[clap(short, long)]
    verbose: bool,

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

    /// Lists Amazon-provided models
    ///
    /// Useful if you want to try another model and need it's model-id or inference-profile-id
    #[clap(short, long)]
    list: bool,

    /// Prefilled assistant response.
    ///
    /// If provided, then when this model is invoked this prompt will be sent to the model for it to use to start off its answer.
    #[clap(short, long)]
    prefill: Option<String>,

    /// Additional media files (images, videos) to attach as context for the model.
    ///
    /// Each file should be specified with its own --attach argument.
    /// Media type will be determined from the file extension.
    ///
    /// Supported formats:
    /// - Images: png, jpg, jpeg, gif, webp (local files only)
    /// - Videos: mp4, mov, mkv, webm, flv, mpeg, mpg, wmv, 3gp (supports both local files and S3 locations via s3://)
    ///
    /// Note: S3 locations (s3://) are only supported for video files.
    #[clap(short, long)]
    attach: Vec<String>,

    /// User prompt.
    ///
    /// The actual user prompt.
    prompt: String,
}

// #[async_std::main]
#[tokio::main]
async fn main() {
    let cli = CliArgs::parse();

    let verbosity = if cli.verbose { 3 } else { 2 };
    stderrlog::new().verbosity(verbosity).init().unwrap();

    if cli.list {
        let cpclient = rusty_bedrock_lib::new_controlplane_client(cli.aws_profile.clone()).await;
        let list = rusty_bedrock_lib::list_models(&cpclient, Some("Amazon".to_string())).await;
        for item in list {
            println!("{}", item);
        }
        return;
    }

    let client = rusty_bedrock_lib::new_runtime_client(cli.aws_profile).await;

    let attachments: Vec<FileReference> = cli.attach.into_iter().map(|s| s.into()).collect();
    let result = nova::text::invoke_model(
        &client,
        cli.model,
        None,
        attachments,
        cli.system,
        cli.prefill,
        cli.prompt,
    )
    .await;

    println!("{}", result.1);
}
