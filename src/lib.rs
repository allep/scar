use std::error::Error;
use use_cases::TopNUseCase;

pub mod dependency_analyzer;
pub mod file;
pub mod project_scanner;
pub mod use_cases;

#[derive(Debug)]
pub struct Config {
    project_path: String,
    is_topn_analyzer: bool,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, Box<dyn Error>> {
        if args.len() < 2 {
            return Err(String::from("Not enough arguments").into());
        }

        // check mode
        if args.len() > 2 {
            Ok(Config {
                project_path: args[1].clone(),
                is_topn_analyzer: args[2] == "-t",
            })
        } else {
            Ok(Config {
                project_path: args[1].clone(),
                is_topn_analyzer: false,
            })
        }
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    if config.is_topn_analyzer {
        TopNUseCase::do_sorted_topn_inclusions(&config.project_path, 10)?;
    }

    Ok(())
}
