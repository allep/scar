use clap::Parser;
use std::error;

#[derive(Parser)]
#[command(name = "scar")]
struct Args {
    #[arg(short = 't', long = "topn")]
    topn_analyzer: bool,

    #[arg(short = 'i', long = "topnimpact")]
    topn_impact_analyzer: bool,

    #[arg(short = 'p', long = "path")]
    project_path: String,

    #[arg(short = 'n', long = "num", default_value = "42")]
    output_size: usize,

    #[arg(short = 'd', long = "debug", default_value = "false")]
    debug: bool,
}

fn main() -> Result<(), Box<dyn error::Error>> {
    println!("--- Source Code Analyzer ---");

    let args = Args::parse();

    let config = scar::Config::build(
        &args.project_path,
        args.topn_analyzer,
        args.topn_impact_analyzer,
        args.output_size,
        args.debug,
    )?;
    scar::run(config)?;

    Ok(())
}
