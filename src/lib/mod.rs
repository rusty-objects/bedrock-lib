//! Support for calling various models
//! See: https://docs.aws.amazon.com/bedrock/latest/userguide/models-supported.html

pub mod file;
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
    ///
    /// base_write_path may end in a filename prefix (and not just a directory)
    /// For example instead of: `/tmp/`, it might be `/tmp/abc123-` with the expectation
    /// that files are written with paths such as `/tmp/abc123-1.jpg`.
    fn render_response(
        &self,
        body: String,
        base_write_path: String,
    ) -> (String, Vec<DownloadLocation>);
}

// the location of any saved assets from the response
pub enum DownloadLocation {
    Image(String),
    Video(String),
}
