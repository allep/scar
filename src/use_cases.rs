use crate::dependency_analyzer::DependencyAnalyzer;
use crate::dependency_analyzer::DependencyEntry;
use crate::project_scanner::ProjectScanner;
use std::cmp::Ordering;
use std::error::Error;
use std::path::Path;

pub struct TopNUseCase {}

impl TopNUseCase {
    pub fn do_sorted_topn_inclusions(path: &str, num: usize) -> Result<(), Box<dyn Error>> {
        let path = Path::new(path);
        let mut project = ProjectScanner::make(path)?;

        let files = project.scan_files()?;
        let analyzer = DependencyAnalyzer::make(&files)?;

        println!("Sorting ...");
        let sorted_inclusions = analyzer.get_sorted_inclusion();
        println!("Sorted!");

        let sorted_inclusions: &[DependencyEntry] = match sorted_inclusions.len().cmp(&num) {
            Ordering::Less | Ordering::Equal => &sorted_inclusions[..],
            Ordering::Greater => &sorted_inclusions[..num],
        };

        for i in sorted_inclusions.iter() {
            println!(
                "Include found: {}, num inclusions: {}",
                i.get_file_name(),
                i.get_including_file_paths().len()
            );
        }

        Ok(())
    }

    pub fn do_sorted_topn_impact(path: &str, num: usize) -> Result<(), Box<dyn Error>> {
        let path = Path::new(path);
        let mut project = ProjectScanner::make(path)?;

        let files = project.scan_files()?;
        let analyzer = DependencyAnalyzer::make(&files)?;

        println!("Sorting impact ...");
        let sorted_impacts = analyzer.get_sorted_impact();

        println!("Sorted!");

        let sorted_impacts: &[DependencyEntry] = match sorted_impacts.len().cmp(&num) {
            Ordering::Less | Ordering::Equal => &sorted_impacts[..],
            Ordering::Greater => &sorted_impacts[..num],
        };

        for i in sorted_impacts.iter() {
            println!(
                "Include found: {}, num impacted files: {}",
                i.get_file_name(),
                i.get_including_file_paths().len()
            );
        }

        Ok(())
    }
}
