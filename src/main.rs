use std::{env, error};

fn main() -> Result<(), Box<dyn error::Error>> {
    println!("--- Source Code Analyzer ---");

    let args: Vec<String> = env::args().collect();
    let config = scar::Config::build(&args)?;
    scar::run(config)?;

    Ok(())
}
