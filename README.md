# gensh
interactive shell for gen ai models

### nova

Invokes the Amazon Nova family of text models, using Bedrock's `InvokeModel` call under the hood.

InvokeModel does not have a unified API across models (takes a Blob `body` input, which is a model specific
json document).

See the Amazon Bedrock user guide for more information:
* https://docs.aws.amazon.com/bedrock/latest/userguide/inference.html
* https://docs.aws.amazon.com/bedrock/latest/userguide/model-parameters.html
* https://docs.aws.amazon.com/bedrock/latest/userguide/models-supported.html
* https://docs.aws.amazon.com/bedrock/latest/userguide/model-access.html

#### usage
```
$ cargo run --bin nova --help
$ cargo build && ./target/debug/nova --verbose --aws-profile bedrock --system "you are a pirate" --assistant "Here is a rhyming answer:" "What should I have for dinner?"
```

### canvas

Invokes the Amazon Nova Canvas image generation model, using Bedrock's `InvokeModel` under the hood.

InvokeModel does not have a unified API across models (takes a Blob `body` input, which is a model specific
json document).  The request schema for Amazon Nova Canvas is different than for the text models.

See the Amazon Bedrock user guide for more information:
* https://docs.aws.amazon.com/nova/latest/userguide/content-generation.html
* https://docs.aws.amazon.com/bedrock/latest/userguide/models-supported.html
* https://docs.aws.amazon.com/bedrock/latest/userguide/model-access.html

#### usage
```
$ cargo run --bin canvas --help
$ cargo build && ./target/debug/canvas -v -p bedrock -o ~/Desktop -n "lily pads" "swan lake"
```

## Setup

To use, you need to have access to an AWS account so you can interact with Amazon Bedrock.  Additionally,
you must [request and obtain access](https://docs.aws.amazon.com/bedrock/latest/userguide/model-access.html)
to the Bedrock models you're interested in using.

You must also have access to Bedrock.  One way to set this up is to get an IAM user with `BedrockFullAccess`
and store their credentials in a `[default]` profile under `~/.aws/credentials`.  The tooling also supports
overriding the default profile name via the `--aws-profile` option.

## Issues
* Converse
    * support doc inputs (word, pdf)
    * anthropic 
        * sonnet and haiku: specifically image output
        * anthropic.claude-3-sonnet-20240229-v1:0
        * https://docs.aws.amazon.com/bedrock/latest/userguide/model-parameters-claude.html
        * https://docs.aws.amazon.com/bedrock/latest/userguide/model-parameters-anthropic-claude-text-completion.html
* RetrieveAndGenerate
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
* APIs
    * [Amazon Bedrock](https://docs.aws.amazon.com/bedrock/latest/APIReference/API_Operations_Amazon_Bedrock.html): Control plane, including batch job invocation and management.
    * [Amazon Bedrock Rutime](https://docs.aws.amazon.com/bedrock/latest/APIReference/API_Operations_Amazon_Bedrock_Runtime.html): Data plane for individual model invocation/conversing, including async invoke.  Also includes guardrail application.
    * [Agents for Amazon Bedrock](https://docs.aws.amazon.com/bedrock/latest/APIReference/API_Operations_Agents_for_Amazon_Bedrock.html): Agent Control plane, including flow APIs and knowledge base APIs
    * [Agents for Amazon Bedrock Runtime](https://docs.aws.amazon.com/bedrock/latest/APIReference/API_Operations_Agents_for_Amazon_Bedrock_Runtime.html): Agent Data plane, including flow APIs
* [Nova User Guide][https://docs.aws.amazon.com/nova/latest/userguide/]