use std::error::Error;
use use_cases::TopNUseCase;

pub mod dependency_analyzer;
pub mod file;
pub mod project_scanner;
pub mod use_cases;

#[derive(Debug)]
pub struct Config<'a> {
    project_path: &'a str,
    mode: ScarMode,
}

#[derive(Debug)]
enum ScarMode {
    TopNAnalisys(usize),
}

impl<'a> Config<'a> {
    pub fn build(
        path: &'a str,
        is_topn: bool,
        output_size: usize,
    ) -> Result<Config<'a>, Box<dyn Error>> {
        if is_topn {
            return Ok(Config {
                project_path: path,
                mode: ScarMode::TopNAnalisys(output_size),
            });
        }

        Err("Invalid input mode.".into())
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    match config.mode {
        ScarMode::TopNAnalisys(output_size) => {
            TopNUseCase::do_sorted_topn_inclusions(config.project_path, output_size)?;
        }
    }

    Ok(())
}
