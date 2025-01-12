use clap::{Parser, Subcommand};
use genlib::{
    amazon::{self, NovaBedrock},
    DownloadLocation,
};

/// Calls InvokeModel for Amazon Bedrock
///
/// You must be opted into the model specified in you AWS account.
///
/// Each model has its own convention for inference parameters, so each
/// supported model is presented as a sub-command to allow for per-model
/// invocation differences.
///
/// See the Amazon Bedrock user guide for more information:
///
/// - https://docs.aws.amazon.com/bedrock/latest/userguide/inference.html
///
/// - https://docs.aws.amazon.com/bedrock/latest/userguide/model-parameters.html
///
/// - https://docs.aws.amazon.com/bedrock/latest/userguide/models-supported.html
///
/// - https://docs.aws.amazon.com/bedrock/latest/userguide/model-access.html
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct CliArgs {
    /// AWS profile override
    ///
    /// AWS region and credentials are selected in the following sequence:
    ///
    /// 1/ Explicit Override:
    /// When this --profile option is specified, the named profile will be read from
    /// ~/.aws/config and ~/.aws/credentials.
    ///
    /// 2/ Environment Variables:
    /// As described here: https://docs.aws.amazon.com/cli/v1/userguide/cli-configure-envvars.html
    ///
    /// 3/ Default profile:
    /// Uses the default profile from ~/.aws/config and ~/.aws/credentials.
    ///
    /// See the AWS docs for more information:
    ///
    /// - https://docs.aws.amazon.com/sdkref/latest/guide/file-format.html
    ///
    /// - https://docs.aws.amazon.com/sdk-for-rust/latest/dg/region.html
    ///
    /// - https://docs.aws.amazon.com/sdk-for-rust/latest/dg/credproviders.html
    #[clap(short = 'p', long)]
    aws_profile: Option<String>,

    /// dumps raw output
    #[clap(short, long)]
    verbose: bool,

    #[clap(subcommand)]
    commands: Commands,
}

// NOTE:
// Don't put rust doc on these enum variants, or else clap derive will
// display those docs in lieu of the ones from each variant's Args impl.
#[derive(Subcommand, Debug)]
enum Commands {
    AmznNovaLite(amazon::NovaLiteArgs),
}

// #[async_std::main]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = CliArgs::parse();

    // Wire up SdkConfig.  Various reading on the subject:
    //
    // https://docs.rs/aws-config/latest/aws_config/
    // https://docs.aws.amazon.com/cli/v1/userguide/cli-configure-files.html
    // https://docs.aws.amazon.com/sdk-for-rust/latest/dg/configure.html
    // https://docs.aws.amazon.com/sdkref/latest/guide/file-format.html
    // https://docs.aws.amazon.com/sdk-for-rust/latest/dg/credproviders.html
    // https://docs.rs/aws-config/latest/aws_config/profile/credentials/struct.ProfileFileCredentialsProvider.html
    // https://docs.rs/aws-config/latest/aws_config/profile/struct.ProfileFileRegionProvider.html
    let config = if let Some(profile) = cli.aws_profile {
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

    // based on the command line args, make a bedrock request serializer
    let bedrock_serde: Box<dyn genlib::BedrockSerde> = match cli.commands {
        Commands::AmznNovaLite(args) => {
            let req: NovaBedrock = args.into();
            Box::new(req)
        }
    };

    if cli.verbose {
        println!(">>> request");
        println!("id: {}", bedrock_serde.model_id());
        println!("{}", bedrock_serde.body());
    }

    // Send InvokeModel to Amazon Bedrock
    //
    // https://docs.rs/aws-sdk-bedrockruntime/latest/aws_sdk_bedrockruntime/struct.Client.html#method.invoke_model
    let result = client
        .invoke_model()
        .content_type("application/json")
        .accept("application/json")
        .model_id(bedrock_serde.model_id())
        .body(bedrock_serde.body().into_bytes().into())
        .send()
        .await;

    // Process the results, pretty printing the output
    match result {
        Ok(result) => {
            let body = result.clone().body;
            let body_bytes = body.as_ref();
            let body_string = String::from_utf8(body_bytes.to_owned()).unwrap();

            if cli.verbose {
                println!("\n<<< response\n{:#?}", result);

                // printing the result will redact the contents of the body, so we print explicitly
                println!("{}\n", body_string);
            }

            let (pretty, locations) =
                bedrock_serde.render_response(body_string, "/tmp/".to_string());
            println!("{}", pretty);

            for location in locations {
                match location {
                    DownloadLocation::Image(loc) => println!("Saved image to: {}", loc),
                    DownloadLocation::Video(loc) => println!("Saved video to: {}", loc),
                }
            }
        }
        Err(result) => println!("\nerror:\n{:#?}", result),
    }

    Ok(())
}
