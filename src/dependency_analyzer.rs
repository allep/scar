use crate::file::File;
use colored::Colorize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;

pub struct DependencyAnalyzer<'a> {
    _files: &'a [File],

    /**
     * The hashmap containing dependencies.
     * - key: the dependency file (e.g., "stdio.h")
     * - value: a set of files directly including the dependency file (e.g., "main.cpp",
     * "foobar.cpp")
     */
    modules_inclusion: HashMap<&'a str, HashSet<&'a str>>,

    debug: bool,
}

impl<'a> DependencyAnalyzer<'a> {
    pub fn make(files: &'a [File], debug: bool) -> Result<DependencyAnalyzer<'a>, Box<dyn Error>> {
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
            debug,
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
    pub fn get_sorted_inclusion(&self) -> Vec<DependencyEntry<'a>> {
        let included_files = self.get_included_files();
        return self.get_sorted_inclusion_impl(included_files);
    }

    pub fn get_sorted_inclusion_no_external(&self) -> Vec<DependencyEntry<'a>> {
        let included_files = self.get_included_files();
        let included_files_no_external = self.filter_outside_inclusions(included_files);
        return self.get_sorted_inclusion_impl(included_files_no_external);
    }

    // impl function for get_sorted_inclusion
    fn get_sorted_inclusion_impl(&self, included_files: Vec<&'a str>) -> Vec<DependencyEntry<'a>> {
        let mut included_files = included_files;

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
        

    pub fn filter_outside_inclusions(&self, included_files: Vec<&'a str>) -> Vec<&'a str> {
        // Remove from included files the ones that are not inside the scanned files
        // Create a HashSet of file names for O(1) lookup
        let file_names: std::collections::HashSet<_> = self._files
            .iter()
            .map(|file| file.get_name())
            .collect();

        included_files
            .into_iter()
            .filter(|&included_file| {
                // Keep files that are NOT in the HashSet
                file_names.contains(included_file)
            })
            .collect()
    }

    /**
     * Returns the list of dependency impacts, i.e., the actual number of files impacted by the
     * current file (considering multiple-levels of inclusions).
     */
    pub fn get_sorted_impact(&self) -> Vec<DependencyEntry> {
        let included_files = self.get_included_files();
        self.get_sorted_impact_impl(included_files)
    }

    /**
     * Returns the list of dependency impacts, i.e., the actual number of files impacted by the
     * current file (considering multiple-levels of inclusions). Removes external files from the analysis.
     */
    pub fn get_sorted_impact_no_external(&self) -> Vec<DependencyEntry> {
        let included_files = self.get_included_files();
        let included_files_no_external = self.filter_outside_inclusions(included_files);
        self.get_sorted_impact_impl(included_files_no_external)
    }

    fn get_sorted_impact_impl(
    &self,
    included_files: Vec<&'a str>,
) -> Vec<DependencyEntry<'a>> {
    let mut dependencies = Vec::new();
    for inc in &included_files {
        match self.dfs_tree(inc) {
            Ok(tree) => {
                if self.debug {
                    tree.print_tree(inc, 0);
                }
                // Only works if visit_order contains references to data with lifetime 'a
                // For example, if visit_order contains references to self._files data
                let filtered_paths: HashSet<&'a str> = tree.visit_order
                    .iter()
                    .filter(|&v| v != inc)
                    .filter_map(|path| {
                        // Find the corresponding reference in self._files with lifetime 'a
                        self._files.iter()
                            .find(|file| file.get_name() == *path)
                            .map(|file| file.get_name())
                    })
                    .collect();
                
                dependencies.push(DependencyEntry {
                    file_name: inc,
                    including_files_paths: filtered_paths,
                });
            }
            Err(e) => println!("Error while computing sorted impact: {}", e),
        }
    }
    dependencies.sort_by(|a, b| {
        b.including_files_paths.len().cmp(&a.including_files_paths.len())
    });
    dependencies
}

    pub fn extract_filename_from_path(path: &str) -> &str {
        match path.split("/").last() {
            Some(last_token) => last_token,
            None => path,
        }
    }

    fn dfs_tree(&self, start_node: &'a str) -> Result<DFSTree, Box<dyn Error>> {
        assert!(!self.modules_inclusion.is_empty());

        if !self.modules_inclusion.contains_key(start_node) {
            return Err(format!("Starting node {} not found.", start_node).into());
        }

        let mut visited = HashSet::new();
        let mut dfs_tree = DFSTree::make();

        fn dfs_recursive<'a>(
            current: &'a str,
            parent: Option<&'a str>,
            adj_list: &HashMap<&'a str, HashSet<&'a str>>,
            visited: &mut HashSet<&'a str>,
            tree: &mut DFSTree<'a>,
        ) {
            visited.insert(current);
            tree.visit_order.push(current);

            if let Some(p) = parent {
                tree.add_edge(p, current);
            }

            if let Some(neighbors) = adj_list.get(current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        dfs_recursive(neighbor, Some(current), adj_list, visited, tree);
                    }
                }
            }
        }

        dfs_recursive(
            start_node,
            None,
            &self.modules_inclusion,
            &mut visited,
            &mut dfs_tree,
        );

        Ok(dfs_tree)
    }

    fn get_included_files(
        &self) -> Vec<&'a str> {
            let included_files: Vec<&str> = self.modules_inclusion.keys().cloned().collect();
            assert!(!included_files.is_empty(), "No included files found.");
            included_files
        }

}

#[derive(Debug, Clone)]
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

#[derive(Debug)]
struct DFSTree<'a> {
    tree: HashMap<&'a str, Vec<&'a str>>,
    visit_order: Vec<&'a str>,
}

impl<'a> DFSTree<'a> {
    fn make() -> Self {
        DFSTree {
            tree: HashMap::new(),
            visit_order: Vec::new(),
        }
    }

    fn add_edge(&mut self, parent: &'a str, child: &'a str) {
        self.tree.entry(parent).or_insert_with(Vec::new).push(child);
    }

    fn print_tree(&self, node: &str, level: usize) {
        let message = format!("{}{}", "    ".repeat(level), node);
        match level % 5{
            0 => println!("{}", message.red()),
            1 => println!("{}", message.yellow()),
            2 => println!("{}", message.green()),
            3 => println!("{}", message.blue()),
            4 => println!("{}", message.purple()),
            _ => unreachable!(),
        }
        if let Some(children) = self.tree.get(node) {
            for child in children {
                self.print_tree(child, level + 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static FIRST_FILE_CONTENT: &str = "\
#include <iostream>
#include \"foobar.h\"
//#include \"commented_out.h\"
/*#include \"another_commented_out.h\"

int main(void) {
    printf(\"Hello world\");
    return 0;
};
    ";

    static SECOND_FILE_CONTENT: &str = "\
#include \"blablah.h\"

class Point {{
    explicit Point() = default;
    virtual ~Point() = default;
}};
    ";

    static THIRD_FILE_CONTENT: &str = "\
#include \"foobar.h\"

namespace Leviathan {

void DoSomeStuff(uint8_t value) {}

}
    ";

    fn create_sample_files() -> Result<Vec<File>, Box<dyn Error>> {
        let first_name = "main.cpp";
        let first = File::make(first_name, FIRST_FILE_CONTENT)?;

        let second_name = "foobar.h";
        let second = File::make(second_name, SECOND_FILE_CONTENT)?;

        let third_name = "leviathan.h";
        let third = File::make(third_name, THIRD_FILE_CONTENT)?;

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

        let debug = true;
        let analyzer = DependencyAnalyzer::make(&files, debug)?;
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

        let debug = true;
        let analyzer = DependencyAnalyzer::make(&files, debug)?;
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

        let debug = true;
        let analyzer = DependencyAnalyzer::make(&files, debug)?;
        let sorted_impacts = analyzer.get_sorted_impact();

        let expected = [
            ("foobar.h", 2),
            ("iostream", 1),
            ("blablah.h", 3),
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
