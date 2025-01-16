use clap::Parser;

/// Lists Bedrock models
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

    /// Optional case-insensitive provider filter, e.g. Amazon, amazon, Anthropic.
    ///
    /// https://docs.aws.amazon.com/bedrock/latest/userguide/models-supported.html
    provider: Option<String>,
}

// #[async_std::main]
#[tokio::main]
async fn main() {
    let cli = CliArgs::parse();

    let cpclient = rusty_bedrock_lib::new_controlplane_client(cli.aws_profile.clone()).await;
    let list = rusty_bedrock_lib::list_models(&cpclient, cli.provider).await;
    for item in list {
        println!("{}", item);
    }
}
