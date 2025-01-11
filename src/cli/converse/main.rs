/*
use std::error::Error;
use std::fmt;
use std::ops::AddAssign;

use async_std::prelude::*;
use clap::Parser;
use rustyline::DefaultEditor;
use shellfish::rustyline::DefaultEditor as DefaultEditorRusty;
use shellfish::{app, clap_command, handler::DefaultAsyncHandler, Command, Shell};

#[macro_use]
extern crate shellfish;

// #[macro_use]
// extern crate clap;

// ------ CLAP -------
/// Simple command to greet a person
///
/// This command will greet the person based of a multitide
/// of option flags, see below.
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct GreetArgs {
    /// Name of the person to greet
    name: String,

    /// Age of the person to greet
    #[clap(short, long)]
    age: Option<u8>,

    /// Whether to be formal or note
    #[clap(short, long)]
    formal: bool,
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define a shell
    let mut shell = Shell::new_with_async_handler(
        (),
        "<[Shellfish Example]>-$ ",
        DefaultAsyncHandler::default(),
        DefaultEditorRusty::new()?,
    );
    shell
        .commands
        .insert("greet", clap_command!((), GreetArgs, greet));
    shell.run_async().await?;

    Ok(())
}

fn greet(_state: &mut (), args: GreetArgs) -> Result<(), Box<dyn std::error::Error>> {
    // .. snip .. /

    Ok(())
}
// ------ CLAP -------

/*
#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define a shell
    let mut shell = Shell::new_with_async_handler(
        0_u64,
        "<[Shellfish Example]>-$ ",
        DefaultAsyncHandler::default(),
        // DefaultEditor::new()?,
        DefaultEditorRusty::new()?,
    );

    // Add some commands
    shell
        .commands
        .insert("greet", Command::new("greets you.".to_string(), greet));

    shell
        .commands
        .insert("echo", Command::new("prints the input.".to_string(), echo));

    shell.commands.insert(
        "count",
        Command::new("increments a counter.".to_string(), count),
    );

    shell.commands.insert(
        "cat",
        Command::new_async(
            "Displays a plaintext file.".to_string(),
            async_fn!(u64, cat),
        ),
    );

    // Check if we have > 2 args, if so no need for interactive shell
    let mut args = std::env::args();
    if args.nth(1).is_some() {
        // Create the app from the shell.
        let mut app = app::App::try_from_async(shell)?;

        // Set the binary name
        app.handler.proj_name = Some("shellfish-example".to_string());
        app.load_cache()?;

        // Run it
        app.run_args_async().await?;
    } else {
        // Run the shell
        shell.run_async().await?;
    }
    Ok(())
}

/// Greets the user
fn greet(_state: &mut u64, args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let arg = args.get(1).ok_or_else(|| Box::new(GreetingError))?;
    println!("Greetings {}, my good friend.", arg);
    Ok(())
}
*/

/// Echos the input
fn echo(_state: &mut u64, args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut args = args.iter();
    args.next();
    for arg in args {
        print!("{} ", arg);
    }
    println!();
    Ok(())
}

/// Acts as a counter
fn count(state: &mut u64, _args: Vec<String>) -> Result<(), Box<dyn Error>> {
    state.add_assign(1);
    println!("You have used this counter {} times", state);
    Ok(())
}

/// Asynchronously reads a file
async fn cat(_state: &mut u64, args: Vec<String>) -> Result<(), Box<dyn Error>> {
    use async_std::fs;

    if let Some(file) = args.get(1) {
        let mut contents = String::new();
        let mut file = fs::File::open(file).await?;
        file.read_to_string(&mut contents).await?;
        println!("{}", contents);
    }

    Ok(())
}

/// Greeting error
#[derive(Debug)]
pub struct GreetingError;

impl fmt::Display for GreetingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "No name specified")
    }
}

impl Error for GreetingError {}

*/

fn main() {
    println!("Hi");
}
