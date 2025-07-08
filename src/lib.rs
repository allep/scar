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
    debug: bool,
}

#[derive(Debug)]
enum ScarMode {
    TopNAnalisys(usize),
    TopNImpactAnalysis(usize),
}

impl<'a> Config<'a> {
    pub fn build(
        path: &'a str,
        is_topn: bool,
        is_impact: bool,
        output_size: usize,
        debug: bool,
    ) -> Result<Config<'a>, Box<dyn Error>> {
        if is_topn {
            return Ok(Config {
                project_path: path,
                mode: ScarMode::TopNAnalisys(output_size),
                debug,
            });
        }

        if is_impact {
            return Ok(Config {
                project_path: path,
                mode: ScarMode::TopNImpactAnalysis(output_size),
                debug,
            });
        }

        Err("Invalid input mode.".into())
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    match config.mode {
        ScarMode::TopNAnalisys(output_size) => {
            let use_case_config =
                use_cases::Config::make(config.project_path, output_size, config.debug);
            TopNUseCase::do_sorted_topn_inclusions(use_case_config)?;
        }
        ScarMode::TopNImpactAnalysis(output_size) => {
            let use_case_config =
                use_cases::Config::make(config.project_path, output_size, config.debug);
            TopNUseCase::do_sorted_topn_impact(use_case_config)?;
        }
    }

    Ok(())
}
