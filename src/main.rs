use std::{env, error};

fn main() -> Result<(), Box<dyn error::Error>> {
    println!("Source Code Analyzer");

    let args: Vec<String> = env::args().collect();
    let config = scar::project::Config::build(&args)?;
    scar::project::run(config)?;

    Ok(())
}
