use crate::dependency_analyzer::DependencyAnalyzer;
use crate::dependency_analyzer::DependencyEntry;
use crate::project_scanner::ProjectScanner;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

fn get_slice_up_to<T>(slice: &[T], num: usize) -> &[T] {
    match slice.len().cmp(&num) {
        Ordering::Less | Ordering::Equal => &slice[..],
        Ordering::Greater => &slice[..num],
    }
}

pub struct TopNUseCase {}

impl TopNUseCase {
    fn make_output_data_from_slice(
        dependency_entries: &[DependencyEntry],
    ) -> HashMap<String, usize> {
        dependency_entries
            .iter()
            .map(|e| {
                (
                    e.get_file_name().to_string(),
                    e.get_including_file_paths().len(),
                )
            })
            .collect()
    }

    /**
     * TopN inclusions use-case
     * Returns the top-N included files by inclusion, i.e., the N most included files in the source
     * tree.
     *
     * - path: the project path to analyze
     * - num: the max number of include to report as output.
     */
    pub fn do_sorted_topn_inclusions(
        config: Config,
    ) -> Result<HashMap<String, usize>, Box<dyn Error>> {
        let path = Path::new(config.path);
        let mut project = ProjectScanner::make(path)?;

        let files = project.scan_files()?;
        let analyzer = DependencyAnalyzer::make(&files, config.debug)?;

        println!("Sorting ...");
        let sorted_inclusions = analyzer.get_sorted_inclusion();
        println!("Sorted!");

        let sorted_inclusions = get_slice_up_to(&sorted_inclusions, config.output_size);
        for i in sorted_inclusions.iter() {
            println!(
                "Source found: {}, num inclusions: {}",
                i.get_file_name(),
                i.get_including_file_paths().len()
            );
        }

        Ok(Self::make_output_data_from_slice(sorted_inclusions))
    }

    /**
     * TopN impact use-case
     * Returns the top-N impacting files by inclusion, i.e., the N most impacting files due to
     * inclusion from the source code tree.
     *
     * - path: the project path to analyze
     * - num: the max number of include to report as output.
     */
    pub fn do_sorted_topn_impact(config: Config) -> Result<HashMap<String, usize>, Box<dyn Error>> {
        let path = Path::new(config.path);
        let mut project = ProjectScanner::make(path)?;

        let files = project.scan_files()?;
        let analyzer = DependencyAnalyzer::make(&files, config.debug)?;

        println!("Sorting impact ...");
        let sorted_impacts = analyzer.get_sorted_impact();

        println!("Sorted!");

        let sorted_impacts: &[DependencyEntry] =
            get_slice_up_to(&sorted_impacts, config.output_size);

        for i in sorted_impacts.iter() {
            println!(
                "Source found: {}, num impacted files: {}",
                i.get_file_name(),
                i.get_including_file_paths().len()
            );
        }

        Ok(Self::make_output_data_from_slice(sorted_impacts))
    }
}

pub struct Config<'a> {
    path: &'a str,
    output_size: usize,
    debug: bool,
}

impl<'a> Config<'a> {
    pub fn make(path: &'a str, output_size: usize, debug: bool) -> Self {
        Config {
            path,
            output_size,
            debug,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_slice_up_to_num_test() {
        let elements = vec![1, 2, 3, 4, 5, 6];

        assert_eq!(vec![1, 2, 3], get_slice_up_to(&elements, 3));
        assert_eq!(vec![1, 2, 3, 4, 5, 6], get_slice_up_to(&elements, 6));
        assert_eq!(vec![1, 2, 3, 4, 5, 6], get_slice_up_to(&elements, 7));
        assert_eq!(vec![1, 2, 3, 4, 5, 6], get_slice_up_to(&elements, 1000));
    }

    #[test]
    fn integration_use_case_inclusion_simple() -> Result<(), Box<dyn Error>> {
        let config = Config::make("tests/simple", 100, false);
        let inclusions = TopNUseCase::do_sorted_topn_inclusions(config)?;
        assert_eq!(4, inclusions.len());
        assert_eq!(2, inclusions["test001.h"]);
        assert_eq!(1, inclusions["test002.h"]);
        assert_eq!(0, inclusions["test001.cpp"]);
        assert_eq!(0, inclusions["test002.cpp"]);
        Ok(())
    }

    #[test]
    fn integration_use_case_impact_simple() -> Result<(), Box<dyn Error>> {
        let config = Config::make("tests/simple", 100, false);
        let impacts = TopNUseCase::do_sorted_topn_impact(config)?;
        assert_eq!(4, impacts.len());
        assert_eq!(3, impacts["test001.h"]);
        assert_eq!(1, impacts["test002.h"]);
        assert_eq!(0, impacts["test001.cpp"]);
        assert_eq!(0, impacts["test002.cpp"]);
        Ok(())
    }
}
