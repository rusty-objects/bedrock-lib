//! Support for calling various models
//! See: https://docs.aws.amazon.com/bedrock/latest/userguide/models-supported.html

pub mod models;
pub use models::amazon;

/// Allows implementations to generate their own inference parameters
pub trait BedrockSerde {
    /// The model id to use in bedrock calls
    fn model_id(&self) -> &str;

    /// The body to use in bedrock calls
    fn body(&self) -> String;

    /// Render the response nicely, possibly also saving
    /// attachments.
    fn render_response(&self, body: String) -> (String, Vec<DownloadLocation>);
}

// the location of any saved assets from the response
pub enum DownloadLocation {
    Image(String),
    Video(String),
}
