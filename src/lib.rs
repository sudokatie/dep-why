pub mod cli;
pub mod config;
pub mod error;
pub mod graph;
pub mod output;
pub mod parsers;

use cli::Args;
pub use error::{Error, Result};

/// Run the dependency tracer with the given arguments
pub fn run(args: Args) -> Result<()> {
    // TODO: implement main logic
    println!("dep-why v{}", env!("CARGO_PKG_VERSION"));
    println!("Package: {:?}", args.package);
    Ok(())
}
