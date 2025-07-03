use dependency_analyzer::DependencyAnalyzer;
use std::error::Error;
use std::path::Path;

pub mod dependency_analyzer;
pub mod file;
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

    // FIXME: this should go in a use-case
    let files = project.scan_files()?;
    let analyzer = DependencyAnalyzer::make(&files)?;

    println!("Sorting ...");
    let sorted_inclusions = analyzer.get_sorted_inclusion();

    println!("Sorted!");
    for i in sorted_inclusions[..50].iter() {
        println!(
            "Include found: {}, num inclusions: {}",
            i.get_file_name(),
            i.get_including_file_paths().len()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {}
