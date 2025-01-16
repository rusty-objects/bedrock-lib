//! Removes some of the boilerplate in creating tools for use with bedrock::Converse
//!
//! ```text
//! let name = "get_weather";
//! let description = "gets the weather for a city for a specified number of days in the future";
//!
//! let inputs = vec![
//!     tool_use::ToolArg::new(
//!         "city",
//!         "city name",
//!         ToolArgType::String,
//!         true,
//!     ) ,
//!     tool_use::ToolArg::new(
//!         "time_horizon",
//!         "for how many days in the future do you want weather?",
//!         ToolArgType::Integer,
//!         true,
//!     ),
//! ];
//!
//! let tool_config = tool_use::mk_tool(name, description, inputs)
//! ```

use std::{collections::HashMap, fmt::Display};

use aws_sdk_bedrockruntime::types::{Tool, ToolConfiguration, ToolInputSchema, ToolSpecification};
use aws_smithy_types::Document;

/// Rust struct representation of a tool's argument.
pub struct ToolArg {
    name: String,
    description: String,
    arg_type: ToolArgType,
    is_mandatory: bool,
}
impl ToolArg {
    pub fn new(
        name: impl ToString,
        description: impl ToString,
        arg_type: ToolArgType,
        is_mandatory: bool,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            arg_type,
            is_mandatory,
        }
    }
}

/// Rust struct representation of a tool's arg's type
pub enum ToolArgType {
    String,
    Integer,
    Float,
    Bool,
    Array,
    // Object, // unclear how to model this in the tool spec
}
impl Display for ToolArgType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolArgType::String => write!(f, "string"),
            ToolArgType::Integer => write!(f, "integer"),
            ToolArgType::Float => write!(f, "float"),
            ToolArgType::Bool => write!(f, "boolean"),
            ToolArgType::Array => write!(f, "array"),
        }
    }
}

// The Rust SDK API for the input schema uses smithy Documents.
//
// Examples and docs on tool encoding:
// https://docs.aws.amazon.com/sdk-for-rust/latest/dg/rust_bedrock-runtime_code_examples.html
// https://github.com/awsdocs/aws-doc-sdk-examples/blob/main/rustv1/examples/bedrock-runtime/src/bin/tool-use.rs#L242
// https://github.com/awsdocs/aws-doc-sdk-examples/blob/main/rustv1/examples/bedrock-runtime/src/bin/tool-use.rs#L50
// https://docs.aws.amazon.com/nova/latest/userguide/tool-use-definition.html
// https://docs.aws.amazon.com/bedrock/latest/userguide/model-parameters-anthropic-claude-messages.html
// https://docs.rs/aws-sdk-bedrockruntime/latest/aws_sdk_bedrockruntime/types/enum.ToolInputSchema.html
//
// Docs have some ambiguity about the json object literally named "json" wrapping the input, but this seems to work
//
// https://docs.rs/aws-sdk-bedrockruntime/latest/aws_sdk_bedrockruntime/types/struct.ToolConfiguration.html
// https://docs.rs/aws-sdk-bedrockruntime/latest/aws_sdk_bedrockruntime/types/struct.ToolSpecification.html
// https://docs.rs/aws-sdk-bedrockruntime/latest/aws_sdk_bedrockruntime/types/enum.ToolInputSchema.html
pub fn mk_tool(
    name: impl ToString,
    description: impl ToString,
    inputs: Vec<ToolArg>,
) -> ToolConfiguration {
    let mut arg_map = HashMap::new();
    let mut required: Vec<Document> = vec![];

    for input in inputs {
        let key = input.name.clone();
        let value = Document::Object(HashMap::from([
            ("type".into(), Document::String(input.arg_type.to_string())),
            ("description".into(), Document::String(input.description)),
        ]));
        arg_map.insert(key, value);
        if input.is_mandatory {
            required.push(input.name.into());
        }
    }

    let input_schema =
        ToolInputSchema::Json(Document::Object(HashMap::<String, Document>::from([
            ("type".into(), Document::String("object".into())),
            ("properties".into(), Document::Object(arg_map)),
            ("required".into(), Document::Array(required)),
        ])));

    let spec = ToolSpecification::builder()
        .name(name.to_string())
        .description(description.to_string())
        .input_schema(input_schema)
        .build()
        .unwrap();
    let tool = Tool::ToolSpec(spec);
    ToolConfiguration::builder().tools(tool).build().unwrap()
}
