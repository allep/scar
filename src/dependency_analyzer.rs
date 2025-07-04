use crate::file::File;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;

pub struct DependencyAnalyzer<'a> {
    _files: &'a [File],
    modules_inclusion: HashMap<&'a str, HashSet<&'a str>>,
}

impl<'a> DependencyAnalyzer<'a> {
    pub fn make(files: &'a [File]) -> Result<DependencyAnalyzer<'a>, Box<dyn Error>> {
        let mut modules_inclusion = HashMap::new();

        for f in files {
            let path = f.get_name();
            let current_file_name = Self::extract_filename_from_path(path);

            let mut dependencies = HashSet::new();
            f.get_used_modules().iter().for_each(|p| {
                let dependency_name = Self::extract_filename_from_path(p);
                dependencies.insert(dependency_name);
            });

            modules_inclusion
                .entry(current_file_name)
                .or_insert(HashSet::new());

            for d in dependencies {
                modules_inclusion
                    .entry(d)
                    .and_modify(|v| {
                        v.insert(path);
                    })
                    .or_insert({
                        let mut s = HashSet::new();
                        s.insert(path);
                        s
                    });
            }
        }

        Ok(DependencyAnalyzer {
            _files: files,
            modules_inclusion,
        })
    }

    pub fn get_inclusion_map(&self) -> &HashMap<&'a str, HashSet<&'a str>> {
        &self.modules_inclusion
    }

    /**
     * Returns the list of direct inclusions for the current file.
     * Useful when the actual number of direct inclusions is needed, without counting for multiple
     * levels of inclusions.
     */
    pub fn get_sorted_inclusion(&self) -> Vec<DependencyEntry> {
        let mut included_files: Vec<&str> = self.modules_inclusion.keys().cloned().collect();
        // decreasing order: from most to least included
        included_files.sort_by(|&a, &b| {
            self.modules_inclusion[b]
                .len()
                .cmp(&self.modules_inclusion[a].len())
        });

        included_files
            .into_iter()
            .map(|f| {
                let file_name = f;
                let including_files_paths = self.modules_inclusion[f].clone();

                DependencyEntry {
                    file_name,
                    including_files_paths,
                }
            })
            .collect()
    }

    /**
     * Returns the list of dependency impacts, i.e., the actual number of files impacted by the
     * current file (considering multiple-levels of inclusions).
     */
    pub fn get_sorted_impact(&self) -> Vec<DependencyEntry> {
        todo!()
    }

    pub fn extract_filename_from_path(path: &str) -> &str {
        match path.split("/").last() {
            Some(last_token) => last_token,
            None => path,
        }
    }
}

#[derive(Debug)]
pub struct DependencyEntry<'a> {
    file_name: &'a str,
    including_files_paths: HashSet<&'a str>,
}

impl<'a> DependencyEntry<'a> {
    pub fn get_file_name(&self) -> &'a str {
        self.file_name
    }

    pub fn get_including_file_paths(&self) -> &HashSet<&'a str> {
        &self.including_files_paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample_files() -> Result<Vec<File>, Box<dyn Error>> {
        let first_name = "main.cpp";
        let first = File::make(
            first_name,
            "\
#include <iostream>
#include \"foobar.h\"
//#include \"commented_out.h\"
/*#include \"another_commented_out.h\"

int main(void) {
    printf(\"Hello world\");
    return 0;
};
",
        )?;

        let second_name = "foobar.h";
        let second = File::make(
            second_name,
            "\
#include \"blablah.h\"

class Point {{
    explicit Point() = default;
    virtual ~Point() = default;
}};
",
        )?;

        let third_name = "leviathan.h";
        let third = File::make(
            third_name,
            "\
#include \"foobar.h\"

namespace Leviathan {

void DoSomeStuff(uint8_t value) {}

}
",
        )?;

        let fourth_name = "blablah.h";
        let fourth = File::make(
            fourth_name,
            "\
namespace BlaBlah {

}
",
        )?;

        Ok(vec![first, second, third, fourth])
    }

    #[test]
    fn simple_parsing_test() -> Result<(), Box<dyn Error>> {
        let files = create_sample_files()?;

        let analyzer = DependencyAnalyzer::make(&files)?;
        let inclusion_map = analyzer.get_inclusion_map();

        assert_eq!(5, inclusion_map.len());

        let expected_main = HashSet::new();
        assert_eq!(expected_main, inclusion_map["main.cpp"]);

        let expected_foobar = HashSet::from(["main.cpp", "leviathan.h"]);
        assert_eq!(expected_foobar, inclusion_map["foobar.h"]);

        let expected_leviathan = HashSet::new();
        assert_eq!(expected_leviathan, inclusion_map["leviathan.h"]);

        let expected_iostream = HashSet::from(["main.cpp"]);
        assert_eq!(expected_iostream, inclusion_map["iostream"]);

        let expected_blablah = HashSet::from(["foobar.h"]);
        assert_eq!(expected_blablah, inclusion_map["blablah.h"]);

        Ok(())
    }

    #[test]
    fn path_parsing_test_simple() {
        let simple_path = "include/foobar.h";
        assert_eq!(
            "foobar.h",
            DependencyAnalyzer::extract_filename_from_path(simple_path)
        );
    }

    #[test]
    fn path_parsing_test_single_token() {
        let simple_path = "foobar.h";
        assert_eq!(
            "foobar.h",
            DependencyAnalyzer::extract_filename_from_path(simple_path)
        );
    }

    #[test]
    fn path_parsing_test_backslash() {
        // Note: C and C++ standards explicitly say that it's UB using something other than '/'.
        // Hence here we assume that our logic has to report the full string slice in case of
        // backslash.
        let simple_path = "include\\foobar.h";
        assert_eq!(
            "include\\foobar.h",
            DependencyAnalyzer::extract_filename_from_path(simple_path)
        );
    }

    #[test]
    fn top_included_sort_test() -> Result<(), Box<dyn Error>> {
        let files = create_sample_files()?;

        let analyzer = DependencyAnalyzer::make(&files)?;
        let sorted_list = analyzer.get_sorted_inclusion();

        let expected = [
            ("foobar.h", 2),
            ("iostream", 1),
            ("blablah.h", 1),
            ("main.cpp", 0),
            ("leviathan.h", 0),
        ];

        assert_eq!(5, sorted_list.len());

        for e in expected.into_iter() {
            assert!(sorted_list
                .iter()
                .any(|entry| entry.file_name == e.0 && entry.including_files_paths.len() == e.1));
        }

        Ok(())
    }

    #[test]
    fn top_impact_sort_test() -> Result<(), Box<dyn Error>> {
        let files = create_sample_files()?;

        let analyzer = DependencyAnalyzer::make(&files)?;
        let sorted_impacts = analyzer.get_sorted_impact();

        let expected = [
            ("foobar.h", 2),
            ("iostream", 1),
            ("blablah.h", 1),
            ("main.cpp", 0),
            ("leviathan.h", 0),
        ];

        assert_eq!(5, sorted_impacts.len());

        for e in expected.into_iter() {
            assert!(sorted_impacts
                .iter()
                .any(|entry| entry.file_name == e.0 && entry.including_files_paths.len() == e.1));
        }

        Ok(())
    }
}
