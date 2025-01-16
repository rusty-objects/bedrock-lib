//! Functions for dealing with multi-media
//!
//! bedrock:Converse strongly types media, which is a burden for callers that often
//! want to just specify a path to some media and let the software figure it out. The
//! functions here help with the rote mapping.

use aws_sdk_bedrockruntime::types::{
    ContentBlock, DocumentBlock, DocumentFormat, DocumentSource, ImageBlock, ImageFormat,
    ImageSource, S3Location, VideoBlock, VideoFormat, VideoSource,
};

use crate::file::FileReference;

pub struct AttachmentPath(pub String);
#[derive(Debug)]
pub struct InvalidPath(pub String);
impl TryFrom<AttachmentPath> for ContentBlock {
    type Error = InvalidPath;

    fn try_from(value: AttachmentPath) -> Result<Self, Self::Error> {
        let path = value.0;
        let file_ref: FileReference = path.into();
        match (file_ref.file_type, file_ref.location) {
            (crate::file::Type::Image, crate::file::Location::Local) => {
                let format = match image_fmt(&file_ref.extension.0) {
                    Some(format) => format,
                    None => {
                        return Err(InvalidPath(file_ref.path));
                    }
                };
                let blob = crate::file::read(&file_ref.path).into();
                let img_src = ImageSource::Bytes(blob);
                let img_block = ImageBlock::builder()
                    .format(format)
                    .source(img_src)
                    .build()
                    .unwrap();
                return Ok(ContentBlock::Image(img_block));
            }
            (crate::file::Type::Video, crate::file::Location::Local) => {
                let format = video_fmt(&file_ref.extension.0);
                let format = match format {
                    Some(fmt) => fmt,
                    None => {
                        return Err(InvalidPath(file_ref.path));
                    }
                };
                let blob = crate::file::read(&file_ref.path).into();
                let vid_src = VideoSource::Bytes(blob);
                let vid_block = VideoBlock::builder()
                    .format(format)
                    .source(vid_src)
                    .build()
                    .unwrap();
                return Ok(ContentBlock::Video(vid_block));
            }
            (crate::file::Type::Video, crate::file::Location::S3) => {
                let format = video_fmt(&file_ref.extension.0);
                let format = match format {
                    Some(fmt) => fmt,
                    None => {
                        return Err(InvalidPath(file_ref.path));
                    }
                };
                let s3loc = S3Location::builder()
                    .uri(file_ref.path.clone())
                    .build()
                    .unwrap();
                let vid_src = VideoSource::S3Location(s3loc);
                let vid_block = VideoBlock::builder()
                    .format(format)
                    .source(vid_src)
                    .build()
                    .unwrap();
                return Ok(ContentBlock::Video(vid_block));
            }
            (crate::file::Type::Document, crate::file::Location::Local) => {
                let format = doc_fmt(&file_ref.extension.0);
                let format = match format {
                    Some(fmt) => fmt,
                    None => {
                        return Err(InvalidPath(file_ref.path));
                    }
                };
                let blob = crate::file::read(&file_ref.path).into();
                let doc_src = DocumentSource::Bytes(blob);
                let doc_block = DocumentBlock::builder()
                    .format(format)
                    .source(doc_src)
                    .name(file_ref.stem.0)
                    .build()
                    .unwrap();
                return Ok(ContentBlock::Document(doc_block));
            }
            _ => {
                return Err(InvalidPath(file_ref.path));
            }
        }
    }
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
