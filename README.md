# gensh
interactive shell for gen ai models

## Setup

To use, you need to have access to an AWS account so you can interact with Amazon Bedrock.  Additionally,
you must [request and obtain access](https://docs.aws.amazon.com/bedrock/latest/userguide/model-access.html)
to the Bedrock models you're interested in using.

You must also have access to Bedrock.  One way to set this up is to get an IAM user with `BedrockFullAccess`
and store their credentials in a `[default]` profile under `~/.aws/credentials`.  The tooling also supports
overriding the default profile name via the `--aws-profile` option.

## Usage Examples

```
$ cargo run --bin ask --help

$ cargo run --bin ask amzn-nova-lite --help

$ cargo run --bin ask --verbose --aws-profile bedrock amzn-nova-lite -s "you are a pirate" -a "Here is a rhyming answer:" "What should I have for dinner?"

$ cargo build && ./target/debug/ask --verbose --aws-profile bedrock amzn-nova-lite -s "you are a pirate" -a "Here is a rhyming answer:" "What should I have for dinner?"
```

## Issues

Not an active issue yet, but the `shellfish` crate currently doesn't work with clap 4.x, since 
the `clap_command` macro calls `CommandFactory::into_app` from 3.x 

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

## TODO
* Amazon Nova
    * ~~support image input~~
    * ~~amzn: support video input~~
    * plumb inference parameters
    * converse
    * support doc inputs (word, pdf)
    * RetrieveAndGenerate (is this only possible with Converse?)
* anthropic 
    * sonnet and haiku: specifically image output
* General
    * pull request for shellfish/clap issue?

## Bedrock Notes
### Docs
* [Bedrock Rust SDK](https://github.com/awslabs/aws-sdk-rust) ([crate](https://github.com/awslabs/aws-sdk-rust))
* [Bedrock API Reference](https://docs.aws.amazon.com/bedrock/latest/APIReference/welcome.html) 
* [Bedrock User Guide](https://docs.aws.amazon.com/bedrock/latest/userguide/)
* APIs
    * [Amazon Bedrock](https://docs.aws.amazon.com/bedrock/latest/APIReference/API_Operations_Amazon_Bedrock.html): Control plane, including batch job invocation and management.
    * [Amazon Bedrock Rutime](https://docs.aws.amazon.com/bedrock/latest/APIReference/API_Operations_Amazon_Bedrock_Runtime.html): Data plane for individual model invocation/conversing, including async invoke.  Also includes guardrail application.
    * [Agents for Amazon Bedrock](https://docs.aws.amazon.com/bedrock/latest/APIReference/API_Operations_Agents_for_Amazon_Bedrock.html): Agent Control plane, including flow APIs and knowledge base APIs
    * [Agents for Amazon Bedrock Runtime](https://docs.aws.amazon.com/bedrock/latest/APIReference/API_Operations_Agents_for_Amazon_Bedrock_Runtime.html): Agent Data plane, including flow APIs
