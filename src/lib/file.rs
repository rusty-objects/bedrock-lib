use base64::prelude::*;
use shellexpand;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

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

/// Expands filenames with ~ and env variables.  Does not turn
/// relative paths into absolute.
pub fn expand(filename: &str) -> String {
    shellexpand::full(filename).unwrap().to_mut().to_string()
}

/// Reads the contents of the specified file into an RFC4648 base64 encoded string
///
/// Filenames support ~ and env variables
pub fn read_base64(filename: &str) -> String {
    let expanded = expand(filename);
    let contents = fs::read(Path::new(expanded.as_str())).unwrap();
    BASE64_STANDARD.encode(contents)
}

/// Writes the binary decoding of the supplied RFC4648 base64 encoded string to the
/// specified file.
///
/// Filenames support ~ and env variables
pub fn write_base64(filename: &str, contents: String) {
    let expanded = expand(filename);
    let decoded = BASE64_STANDARD.decode(contents).unwrap();
    let _ = fs::write(Path::new(expanded.as_str()), decoded).unwrap();
}
