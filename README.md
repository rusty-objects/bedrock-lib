# bedrock-lib
* Rust struct data model for `bedrock::InvokeModel` Amazon Nova text and Canvas models, including image/video input.
* Rust struct data models for `bedrock::Converse` with support for image, video, and docuemnt input, and basic support for tool usage.
* CLI references using the libraries (no tool usage)

clones and unwraps like crazy

## Usage
```
$ cargo build
$ PATH=$PATH:./target/debug/
$
$ nova --help # interact with Amazon Nova text models
$ nova --verbose --aws-profile bedrock --system "you are a pirate" --assistant "Here is a rhyming answer:" "What should I have for dinner?"
$
$ canvas --help # interact with Amazon Canvas
$ canvas --negative "lily pads" "swan lake"
$
$ converse --help # Have an interactive conversation with the model of your choice
$ converse -v -aws-profile bedrock -s "system prompt for the entire conversation"
$
$ models --help # List foundational models with on demand invocation support
$ models anthropic
```

## Setup

### Rust
You must have [Rust installed](https://www.rust-lang.org/tools/install).

### Amazon Bedrock
To use, you need to have access to an AWS account so you can interact with Amazon Bedrock.  Additionally,
you must [request and obtain access](https://docs.aws.amazon.com/bedrock/latest/userguide/model-access.html)
to the Bedrock models you're interested in using.

You must also have access to Bedrock.  One way to set this up is to get an IAM user with `BedrockFullAccess`
and store their credentials in a `[default]` profile under `~/.aws/credentials`.  The tooling also supports
overriding the default profile name via the `--aws-profile` option.

## TODO
* There are calls to `unwrap` abound, so if something goes wrong, behavior is often a panic.   If this becomes anything serious will need to get serious about gracefully handling Error conditions.
* Add RAG support via RetrieveAndGenerate
* Create a separate CLI that demonstrates tool usage (e.g. canvas can be the tool).
* Submit issue for shellfish/clap issue where the `shellfish` crate currently doesn't work with clap 4.x, since the `clap_command` macro calls `CommandFactory::into_app` from 3.x 
    * https://docs.rs/clap/latest/clap/index.html#modules
    * https://docs.rs/clap/3.2.16/clap/trait.CommandFactory.html

The error with 4.x:

```
error[E0576]: cannot find method or associated constant `into_app` in trait `clap::CommandFactory`
  --> src/cli/gen/main.rs:44:26
   |
44 |         .insert("greet", clap_command!((), GreetArgs, greet));
   |                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in `clap::CommandFactory`
   |
   = note: this error originates in the macro `clap_command` (in Nightly builds, run with -Z macro-backtrace for more info)
```

## Bedrock Notes

* `InvokeModel` is Bedrock's low-level invocation API.  It takes a model-id and a body, which is a freeform json document that's model specific.
* `Converse` is Bedrock's normalized invocation API, which uses a common data model for invoking across models.  Not all models can be invoked with converse, and for those that can not all features are supported.
* `RetrieveAndGenerate` is Bedrock's RAG implementation that queries a knowledge base to aid in generation results.  There is also a `Retrieve` API that only queries the knowledge base and leaves the rest up to the developerhttps://docs.rs/aws-sdk-bedrockagentruntime/latest/aws_sdk_bedrockagentruntime/struct.Client.html#method.retrieve_and_generate.

### Docs
* [Bedrock Rust SDK](https://github.com/awslabs/aws-sdk-rust) ([crate](https://github.com/awslabs/aws-sdk-rust))
* [Bedrock API Reference](https://docs.aws.amazon.com/bedrock/latest/APIReference/welcome.html) 
* [Bedrock User Guide](https://docs.aws.amazon.com/bedrock/latest/userguide/)
    * https://docs.aws.amazon.com/bedrock/latest/userguide/inference.html
    * https://docs.aws.amazon.com/bedrock/latest/userguide/model-parameters.html
    * https://docs.aws.amazon.com/bedrock/latest/userguide/models-supported.html
    * https://docs.aws.amazon.com/bedrock/latest/userguide/model-access.html
* [Nova User Guide][https://docs.aws.amazon.com/nova/latest/userguide/]
    * * https://docs.aws.amazon.com/nova/latest/userguide/content-generation.html
* APIs
    * [Amazon Bedrock](https://docs.aws.amazon.com/bedrock/latest/APIReference/API_Operations_Amazon_Bedrock.html): Control plane, including batch job invocation and management.
    * [Amazon Bedrock Rutime](https://docs.aws.amazon.com/bedrock/latest/APIReference/API_Operations_Amazon_Bedrock_Runtime.html): Data plane for individual model invocation/conversing, including async invoke.  Also includes guardrail application.
    * [Agents for Amazon Bedrock](https://docs.aws.amazon.com/bedrock/latest/APIReference/API_Operations_Agents_for_Amazon_Bedrock.html): Agent Control plane, including flow APIs and knowledge base APIs
    * [Agents for Amazon Bedrock Runtime](https://docs.aws.amazon.com/bedrock/latest/APIReference/API_Operations_Agents_for_Amazon_Bedrock_Runtime.html): Agent Data plane, including flow APIs
