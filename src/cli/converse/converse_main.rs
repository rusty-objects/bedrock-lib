//! Converse cli
//!
//! See:
//! https://docs.aws.amazon.com/nova/latest/userguide/using-converse-api.html
//! https://docs.aws.amazon.com/bedrock/latest/userguide/conversation-inference-call.html
//! https://docs.aws.amazon.com/bedrock/latest/userguide/conversation-inference-examples.html
//! https://docs.rs/aws-sdk-bedrockruntime/latest/aws_sdk_bedrockruntime/operation/converse/builders/struct.ConverseFluentBuilder.html

use aws_sdk_bedrockruntime::types::{
    ContentBlock, ConversationRole, ConverseOutput, Message, SystemContentBlock,
};
use aws_sdk_bedrockruntime::Client;
use clap::Parser;
use log::{debug, warn};
use rusty_bedrock_lib::converse::modalities::{AttachmentPath, InvalidPath};
use shellfish::rustyline::DefaultEditor as DefaultEditorRusty;
use shellfish::{clap_command, handler::DefaultAsyncHandler, Shell};

/// Hold a multi-turn interactive conversation with a model
///
/// Callers need permission for `bedrock:InvokeModel`
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
    #[clap(long)]
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli: CliArgs = CliArgs::parse();

    let verbosity = if cli.verbose { 3 } else { 2 };
    stderrlog::new().verbosity(verbosity).init().unwrap();

    let client = rusty_bedrock_lib::new_runtime_client(cli.aws_profile).await;

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
    for path in args.attach {
        let attachment_path = AttachmentPath(path);
        let content_block = match attachment_path.try_into() {
            Ok(content_block) => content_block,
            Err(InvalidPath(path)) => {
                println!("Invalid attachment path, aborting turn. path: {}", path);
                return Ok(());
            }
        };
        msg_builder = msg_builder.content(content_block);
    }

    // ------- construct message --------
    let new_msg = msg_builder.build().unwrap();
    if state.verbose {
        debug!("model: {}", state.model);
        debug!("{:?}", new_msg);
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

    debug!("{:?}", conversation);

    // ===========================
    // Process response, add assistant's response onto the message history state
    // ===========================
    if let Some(ConverseOutput::Message(msg)) = conversation.output() {
        assert_eq!(&ConversationRole::Assistant, msg.role());
        debug!("{:?}", msg);
        for content in msg.content() {
            match content {
                ContentBlock::Document(_document_block) => todo!(),
                ContentBlock::GuardContent(_guardrail_converse_content_block) => {
                    warn!("-- guardrail --")
                }
                ContentBlock::Image(_image_block) => warn!("-- image --"),
                ContentBlock::Text(s) => warn!("{}", s),
                ContentBlock::ToolResult(_tool_result_block) => warn!("-- tool result --"),
                ContentBlock::ToolUse(_tool_use_block) => warn!("-- tool use --"),
                ContentBlock::Video(_video_block) => warn!("-- video --"),
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
