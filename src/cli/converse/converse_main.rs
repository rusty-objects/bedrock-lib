//! Converse cli
//!
//! See:
//! https://docs.aws.amazon.com/nova/latest/userguide/using-converse-api.html
//! https://docs.aws.amazon.com/bedrock/latest/userguide/conversation-inference-call.html
//! https://docs.aws.amazon.com/bedrock/latest/userguide/conversation-inference-examples.html
//! https://docs.rs/aws-sdk-bedrockruntime/latest/aws_sdk_bedrockruntime/operation/converse/builders/struct.ConverseFluentBuilder.html

use std::process::exit;

use aws_sdk_bedrockruntime::types::{
    ContentBlock, ConversationRole, ConverseOutput, DocumentBlock, DocumentFormat, DocumentSource,
    ImageBlock, ImageFormat, ImageSource, Message, S3Location, SystemContentBlock, VideoBlock,
    VideoFormat, VideoSource,
};
use aws_sdk_bedrockruntime::Client;
use clap::Parser;
use shellfish::rustyline::DefaultEditor as DefaultEditorRusty;
use shellfish::{clap_command, handler::DefaultAsyncHandler, Shell};

/// Hold a multi-turn interactive conversation with a model
///
/// Callers need permission for `InvokeModel`
///
/// Example:
///     converse -p bedrock -o ~/Desktop -m us.amazon.nova-lite-v1:0
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
    #[clap(short = 'p', long)]
    aws_profile: Option<String>,

    /// Whether output should be verbose
    #[clap(short, long)]
    verbose: bool,

    /// Model or inference profile id to use
    ///
    /// Not all models support Converse.  Some models such as those in the Amazon
    /// Nova family are accessible in some Regions only through cross-region inference.
    /// For those, specify an inference profile id.  For example:
    ///
    /// Amazon Nova Lite:
    ///   model-id: amazon.nova-lite-v1:0
    ///   inference-profile-id: us.amazon.nova-lite-v1:0
    ///
    /// Anthropic Claude Sonnet v2
    ///   model-id: anthropic.claude-3-5-sonnet-20241022-v2:0
    ///   inference-profile-id: us.anthropic.claude-3-5-sonnet-20241022-v2:0
    ///
    /// See:
    ///   https://docs.aws.amazon.com/bedrock/latest/userguide/models-supported.html
    ///   https://docs.aws.amazon.com/bedrock/latest/userguide/conversation-inference-supported-models-features.html
    ///   https://docs.aws.amazon.com/bedrock/latest/userguide/models-regions.html
    #[clap(
        short,
        long,
        default_value = "us.anthropic.claude-3-5-sonnet-20241022-v2:0",
        verbatim_doc_comment
    )]
    model: String,

    /// System prompt for the entire conversation
    #[clap(short, long)]
    system: Option<String>,

    /// List models enabled for your account
    ///
    /// https://docs.aws.amazon.com/bedrock/latest/APIReference/API_ListFoundationModels.html
    #[clap(short, long)]
    list: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli: CliArgs = CliArgs::parse();

    if cli.list {
        println!("List of available models:");
        println!("  TODO");
        // https://docs.aws.amazon.com/bedrock/latest/APIReference/API_ListFoundationModels.html
        exit(0);
    }

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

    let system_prompt = cli.system.map(|sys| vec![SystemContentBlock::Text(sys)]);

    let state = ConversationState {
        model: cli.model.clone(),
        client,
        verbose: cli.verbose,
        system_prompt,
        messages: vec![],
    };

    println!("");
    // Define a shell
    let mut shell = Shell::new_with_async_handler(
        state,
        format!("[{}]\n> ", cli.model),
        DefaultAsyncHandler::default(),
        DefaultEditorRusty::new()?,
    );
    shell
        .commands
        .insert("say", clap_command!(ConversationState, SayArgs, async say));
    shell.run_async().await?;

    Ok(())
}

#[derive(Debug)]
pub struct ConversationState {
    pub model: String,
    pub client: Client, // bedrock client
    pub verbose: bool,
    pub system_prompt: Option<Vec<SystemContentBlock>>,
    pub messages: Vec<Message>,
}

/// Send a message to the model
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct SayArgs {
    /// Additional media files (images, videos, documents) to attach as context for the model.
    ///
    /// Each file should be specified with its own --attach argument.  Media type will be determined from the file extension.
    ///
    /// Supported formats:
    /// - Images: png, jpg, jpeg, gif, webp (local files only)
    /// - Videos: mp4, mov, mkv, webm, flv, mpeg, mpg, wmv, 3gp (supports both local files and S3 locations via s3://)
    /// - Documents: csv, doc, docx, html, md, pdf, txt, xls, xlsx (local files only)
    ///
    /// Note: S3 locations (s3://) are only supported for video files.
    /// Note: Not all models support all modalities.
    #[clap(short, long)]
    attach: Vec<String>,

    /// The prompt for your next turn in the conversation
    prompt: String,
}

async fn say(
    state: &mut ConversationState,
    args: SayArgs,
) -> Result<(), Box<dyn std::error::Error>> {
    // ===========================
    // Create a new message from SayArgs
    // with the prompt and attachments
    // ===========================
    let mut msg_builder = Message::builder().role(ConversationRole::User);

    // ---- prompt ----
    msg_builder = msg_builder.content(ContentBlock::Text(args.prompt));

    // --- add attachments ---
    // have to figure out what and where the path refers to
    for path in args.attach {
        let (ftype, floc, stem, ext) = genlib::file::detect(path.clone());
        match (ftype, floc) {
            (genlib::file::Type::Image, genlib::file::Location::Local) => {
                let format = match image_fmt(&ext.0) {
                    Some(format) => format,
                    None => {
                        println!("invalid image format {}, aborting message", path);
                        return Ok(());
                    }
                };
                let blob = genlib::file::read(&path).into();
                let img_src = ImageSource::Bytes(blob);
                let img_block = ImageBlock::builder()
                    .format(format)
                    .source(img_src)
                    .build()
                    .unwrap();
                msg_builder = msg_builder.content(ContentBlock::Image(img_block));
            }
            (genlib::file::Type::Video, genlib::file::Location::Local) => {
                let format = video_fmt(&ext.0);
                let format = match format {
                    Some(fmt) => fmt,
                    None => {
                        println!("invalid video format {}, aborting message", path);
                        return Ok(());
                    }
                };
                let blob = genlib::file::read(&path).into();
                let vid_src = VideoSource::Bytes(blob);
                let vid_block = VideoBlock::builder()
                    .format(format)
                    .source(vid_src)
                    .build()
                    .unwrap();
                msg_builder = msg_builder.content(ContentBlock::Video(vid_block));
            }
            (genlib::file::Type::Video, genlib::file::Location::S3) => {
                let format = video_fmt(&ext.0);
                let format = match format {
                    Some(fmt) => fmt,
                    None => {
                        println!("invalid video format {}, abbording message", path);
                        return Ok(());
                    }
                };
                let s3loc = S3Location::builder().uri(path.clone()).build().unwrap();
                let vid_src = VideoSource::S3Location(s3loc);
                let vid_block = VideoBlock::builder()
                    .format(format)
                    .source(vid_src)
                    .build()
                    .unwrap();
                msg_builder = msg_builder.content(ContentBlock::Video(vid_block));
            }
            (genlib::file::Type::Document, genlib::file::Location::Local) => {
                let format = doc_fmt(&ext.0);
                let format = match format {
                    Some(fmt) => fmt,
                    None => {
                        println!("invalid doc format {}, aborting message", path);
                        return Ok(());
                    }
                };
                let blob = genlib::file::read(&path).into();
                let doc_src = DocumentSource::Bytes(blob);
                let doc_block = DocumentBlock::builder()
                    .format(format)
                    .source(doc_src)
                    .name(stem.0)
                    .build()
                    .unwrap();
                msg_builder = msg_builder.content(ContentBlock::Document(doc_block));
            }
            _ => {
                println!("Unsupported file type: {}", path);
                return Ok(());
            }
        }
    }

    // ------- construct message --------
    let new_msg = msg_builder.build().unwrap();
    if state.verbose {
        println!("model: {}", state.model);
        println!(">> new_msg:\n{:?}", new_msg);
    }
    state.messages.push(new_msg);

    // ===========================
    // Send request to bedrock with entire conversation history
    // ===========================
    let conversation = state
        .client
        .converse()
        .model_id(state.model.clone())
        .set_system(state.system_prompt.clone())
        .set_messages(Some(state.messages.clone()))
        .send()
        .await
        .unwrap();
    if state.verbose {
        println!("<< output:\n{:?}", conversation);
    }

    // ===========================
    // Process response, including assistant's response onto the message history state
    // ===========================
    if let Some(ConverseOutput::Message(msg)) = conversation.output() {
        assert_eq!(&ConversationRole::Assistant, msg.role());
        if state.verbose {
            println!("<< new_msg:\n{:?}", msg);
        }
        for content in msg.content() {
            match content {
                ContentBlock::Document(_document_block) => todo!(),
                ContentBlock::GuardContent(_guardrail_converse_content_block) => {
                    println!("-- guardrail --")
                }
                ContentBlock::Image(_image_block) => println!("-- image --"),
                ContentBlock::Text(s) => println!("{}", s),
                ContentBlock::ToolResult(_tool_result_block) => println!("-- tool result --"),
                ContentBlock::ToolUse(_tool_use_block) => println!("-- tool use --"),
                ContentBlock::Video(_video_block) => println!("-- video --"),
                _ => panic!("Unknown response ContentBlock: {:?}", content),
            }
        }

        // Add the response to the tail of the conversation for the next turn
        state.messages.push(msg.clone())
    } else {
        panic!("No output??");
    };

    Ok(())
}

// https://docs.rs/aws-sdk-bedrockruntime/latest/aws_sdk_bedrockruntime/types/enum.VideoFormat.html
fn video_fmt(format: &str) -> Option<VideoFormat> {
    return match format {
        "flv" => Some(VideoFormat::Flv),
        "mkv" => Some(VideoFormat::Mkv),
        "mov" => Some(VideoFormat::Mov),
        "mp4" => Some(VideoFormat::Mp4),
        "mpg" => Some(VideoFormat::Mpg),
        "mpeg" => Some(VideoFormat::Mpeg),
        "3gp" => Some(VideoFormat::ThreeGp),
        "webm" => Some(VideoFormat::Webm),
        "wmv" => Some(VideoFormat::Wmv),
        _ => None,
    };
}

// https://docs.rs/aws-sdk-bedrockruntime/latest/aws_sdk_bedrockruntime/types/enum.ImageFormat.html
fn image_fmt(format: &str) -> Option<ImageFormat> {
    return match format.to_lowercase().as_str() {
        "gif" => Some(ImageFormat::Gif),
        "jpeg" | "jpg" => Some(ImageFormat::Jpeg),
        "png" => Some(ImageFormat::Png),
        "webp" => Some(ImageFormat::Webp),
        _ => None,
    };
}

// https://docs.rs/aws-sdk-bedrockruntime/latest/aws_sdk_bedrockruntime/types/enum.DocumentFormat.html
fn doc_fmt(format: &str) -> Option<DocumentFormat> {
    return match format.to_lowercase().as_str() {
        "csv" => Some(DocumentFormat::Csv),
        "doc" => Some(DocumentFormat::Doc),
        "docx" => Some(DocumentFormat::Docx),
        "html" => Some(DocumentFormat::Html),
        "md" => Some(DocumentFormat::Md),
        "pdf" => Some(DocumentFormat::Pdf),
        "txt" => Some(DocumentFormat::Txt),
        "xls" => Some(DocumentFormat::Xls),
        "xlsx" => Some(DocumentFormat::Xlsx),
        _ => None,
    };
}
