use std::error::Error;
use std::path::Path;

pub mod file;
pub mod modules_analyzer;
pub mod project_scanner;
pub mod use_cases;

#[derive(Debug)]
pub struct Config {
    project_path: String,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, Box<dyn Error>> {
        if args.len() < 2 {
            return Err(String::from("Not enough arguments").into());
        }

        Ok(Config {
            project_path: args[1].clone(),
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let path = Path::new(&config.project_path);
    let mut project = project_scanner::ProjectScanner::make(path)?;

    project.scan_files()?;

    Ok(())
}

#[cfg(test)]
mod tests {}
