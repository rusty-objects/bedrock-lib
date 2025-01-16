use base64::prelude::*;
use shellexpand;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

/// Wrapper around an RFC4648 Base64 encoded String, accessible via as_ref().
pub struct Base64Encoding(String);
impl Base64Encoding {
    pub fn new(input: String) -> Self {
        Self(input)
    }

    pub fn encode(data: Vec<u8>) -> Self {
        Self(BASE64_STANDARD.encode(data))
    }

    fn decode(self) -> Vec<u8> {
        BASE64_STANDARD.decode(self.0).unwrap()
    }

    pub fn unwrap(self) -> String {
        self.0
    }
}

/// Gets the file extension for the specified path.
///
/// Filenames support ~ and env variables.
pub fn get_extension_from_filename(filename: &str) -> String {
    let expanded = expand(filename);
    Path::new(expanded.as_str())
        .extension()
        .and_then(OsStr::to_str)
        .map(str::to_string)
        .unwrap_or_default()
}

/// Gets the file stem for the specified path
///
/// Filenames support ~ and env variables.
pub fn get_file_stem(filename: &str) -> String {
    let expanded = expand(filename);
    Path::new(expanded.as_str())
        .file_stem()
        .and_then(OsStr::to_str)
        .map(str::to_string)
        .unwrap_or_default()
}

/// Expands filenames with ~ and env variables.  Does not turn
/// relative paths into absolute.
pub fn expand(filename: &str) -> String {
    shellexpand::full(filename).unwrap().to_mut().to_string()
}

/// Reads the contents of the specified file into an RFC4648 base64 encoded string
///
/// Filenames support ~ and env variables
pub fn read_base64(filename: &str) -> Base64Encoding {
    let expanded = expand(filename);
    let contents = fs::read(Path::new(expanded.as_str())).unwrap();
    Base64Encoding::encode(contents)
}

/// Reads the contents of the specified file into an RFC4648 base64 encoded string
///
/// Filenames support ~ and env variables
pub fn read(filename: &str) -> Vec<u8> {
    let expanded = expand(filename);
    fs::read(Path::new(expanded.as_str())).unwrap()
}

/// Writes the binary decoding of the supplied RFC4648 base64 encoded string to the
/// specified file.
///
/// Filenames support ~ and env variables
pub fn write_base64(filename: &str, contents: Base64Encoding) {
    let expanded = expand(filename);
    let decoded = contents.decode();
    let _ = fs::write(Path::new(expanded.as_str()), decoded).unwrap();
}

/// Writes the supplied utf-8 string to the specified file
///
/// Filenames support ~ and env variables
pub fn write_string(filename: &str, contents: String) {
    let expanded = expand(filename);
    let _ = fs::write(Path::new(expanded.as_str()), contents);
}

pub enum Location {
    Local,
    S3,
}

pub enum Type {
    Image,
    Video,
    Document,
}

pub struct FileStem(pub String);
pub struct FileExtension(pub String);

pub struct FileReference {
    pub file_type: Type,
    pub location: Location,
    pub path: String,
    pub stem: FileStem,
    pub extension: FileExtension,
}

impl From<String> for FileReference {
    fn from(value: String) -> Self {
        // Determine location based on path prefix
        let location = if value.starts_with("s3://") {
            Location::S3
        } else {
            Location::Local
        };

        // Get file stem and extension
        let stem = FileStem(get_file_stem(&value).to_lowercase());
        let extension = FileExtension(get_extension_from_filename(&value));

        // Determine file type based on extension
        let file_type = match extension.0.to_lowercase().as_str() {
            // Image formats
            "png" | "jpg" | "jpeg" | "gif" | "webp" => Type::Image,

            // Video formats
            "mp4" | "mov" | "webm" | "mpeg" | "mpg" | "m4v" | "avi" => Type::Video,

            // Document formats
            "csv" | "doc" | "docx" | "html" | "md" | "pdf" | "txt" | "xls" | "xlsx" => {
                Type::Document
            }

            _ => panic!("Unsupported file type {}", value),
        };

        FileReference {
            file_type,
            location,
            path: value,
            stem,
            extension,
        }
    }
}

#[test]
fn extension() {
    let file = "/tmp/foo.bar";
    assert_eq!("bar", get_extension_from_filename(file));
    assert_eq!("foo", get_file_stem(file));

    let file = "/tmp/foo";
    assert_eq!("", get_extension_from_filename(file));
    assert_eq!("foo", get_file_stem(file));

    let file = "s3://bucket/file.baz";
    assert_eq!("baz", get_extension_from_filename(file));
    assert_eq!("file", get_file_stem(file));

    let file = "s3://bucket/file";
    assert_eq!("", get_extension_from_filename(file));
    assert_eq!("file", get_file_stem(file));
}
