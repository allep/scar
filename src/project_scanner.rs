use crate::file::File;
use std::error::Error;
use std::fs::read_to_string;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};
use lazy_static;
use std::collections::HashMap;


lazy_static::lazy_static! {
    static ref CONFIG: HashMap<String, Vec<String>> = {
        let mut config_map = HashMap::new();
        config_map.insert(
            String::from("white_list"),
            vec![
                String::from("Source"),
            ],
        );
        config_map.insert(
            String::from("black_list"),
            vec![
                String::from("Intermediate"),
                String::from("Plugins"),
                String::from("Binaries"),
            ],
        );
        config_map
    };
}

pub struct ProjectScanner<'a> {
    base_path: &'a Path,
    processed_files: u64,
}

impl<'a> ProjectScanner<'a> {
    pub fn make(base_path: &Path) -> Result<ProjectScanner, Box<dyn Error>> {
        Ok(ProjectScanner {
            base_path: base_path,
            processed_files: 0u64,
        })
    }

    pub fn scan_files(&mut self) -> Result<Vec<File>, Box<dyn Error>> {
        let walker = WalkDir::new(&self.base_path).into_iter();
        let mut files = Vec::new();
        for entry in walker.filter_entry(|e| Self::is_valid_entry(e)) {
            let entry = entry?;
            let path = entry.path();
            let file_type = entry.file_type();

            if file_type.is_file() {
                match read_to_string(path) {
                    Ok(content) => {
                        files.push(File::make(path.to_str().unwrap(), &content)?);

                        self.on_processed_file();
                    }
                    Err(error) => {
                        println!(
                            "Error while reading {:#?}. Error = {:?}. Skipping it.",
                            path.to_str(),
                            error
                        );
                    }
                }
            }
        }

        Ok(files)
    }

    fn is_valid_entry(entry: &DirEntry) -> bool {
        entry.file_type().is_dir()
            || (entry
                .file_name()
                .to_str()
                .map(|s| Self::is_valid_file_path(s))
                .unwrap_or(false)
        && !Self::is_blacklisted(entry))
        //&& Self::is_whitelisted(entry)
    }

    fn is_valid_file_path(path: &str) -> bool {
        !path.starts_with(".") && (path.ends_with(".cpp") || path.ends_with(".h"))
    }

    fn on_processed_file(&mut self) {
        self.processed_files += 1;
        if self.processed_files > 0 && self.processed_files % 1000 == 0 {
            println!("Processed num. files: {}", self.processed_files);
        }
    }

    fn is_blacklisted(entry: &DirEntry) -> bool {
        let path = entry.path().to_str().map(|s| s.to_string()).unwrap_or_default();

        let is_blacklisted = CONFIG
            .get("black_list")
            .map_or(false, |black_list| {
                black_list.iter().any(|bl| path.contains(bl))
            });

        is_blacklisted
    }

    fn is_whitelisted(entry: &DirEntry) -> bool {
        let path = entry.path().to_str().map(|s| s.to_string()).unwrap_or_default();

        CONFIG
            .get("white_list")
            .map_or(true, |white_list| {
                white_list.iter().all(|wl| path.contains(wl))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path;
    use std::path::PathBuf;
    use tempdir::TempDir;
    use std::path::Path;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref TEST_PATH: PathBuf = PathBuf::from("/media/workspace/");
        static ref INVALID_TEST_PATH: PathBuf = PathBuf::from(".media/workspace/");
    
        static ref FIRST_TEST_CONTENT: String = "
        #include \"third.h\"
        #include \"very_basic_header.h\"
        
        void foobar() {{
            // doing some internal stuff here
            }}".to_string();
            
            static ref SECOND_TEST_CONTENT: String = "#include <iostream>
            #include \"third.h\"
            // #include \"some_random_header.h\"

            void main() {{
                // commented out code
            }}".to_string();
            
            static ref THIRD_TEST_CONTENT: String = "
            #include \"some_random_header_too.h\"
            
            class FooBar {{
                explicit FooBar() = default;
                
                void DoStuff() noexcept {{}};
                }};".to_string();
   }

    fn create_file(
        path: &Path,
        name: &str,
        content: &str,
    ) -> Result<std::fs::File, Box<dyn Error>> {
        let full_path = path.join(name);
        let mut file = std::fs::File::create(full_path)?;

        writeln!(file, "{}", content)?;
        Ok(file)
    }

    fn create_cpp_files_in_path(path: &Path, files: Vec<&str>, contents: Vec<&str>) -> Result<Vec<File>, Box<dyn Error>> {

        assert!(files.len() == contents.len(), "Files and contents must have the same length");
        let mut created_files = Vec::new();

        for (file, content) in files.iter().zip(contents.iter()) {
            let created_file = create_file(path, file, content)?;
            created_files.push(created_file);
        }

        Ok(created_files)
    }

    fn create_dir_tree() -> Result<(TempDir, TempDir), Box<dyn Error>> {
        let temp_dir = TempDir::new("scar_cpp_directory_parsing_test")
            .map_err(|_| "Temporary directory creation error")?;

        let inner_temp_dir =
            TempDir::new_in(temp_dir.path(), "scar_cpp_inner_directory_parsing_test")
                .map_err(|_| "Temporary inner directory creation error")?;

        Ok((temp_dir, inner_temp_dir))
    }

    #[test]
    fn cpp_directory_parsing_test() -> Result<(), Box<dyn Error>> {
        // arrange

        let (temp_base_dir, temp_inner_dir) = create_dir_tree()?;

        let first_level_files = create_cpp_files_in_path(temp_base_dir.path(), vec!["first.cpp", "second.cpp", "third.h"], vec![&FIRST_TEST_CONTENT, &SECOND_TEST_CONTENT, &THIRD_TEST_CONTENT])?;
        let second_level_files = create_cpp_files_in_path(temp_inner_dir.path(), vec!["first.cpp", "second.cpp", "third.h"], vec![&FIRST_TEST_CONTENT, &SECOND_TEST_CONTENT, &THIRD_TEST_CONTENT])?;

        let mut project = super::ProjectScanner::make(&temp_base_dir.path())?;

        // act
        let files = project.scan_files()?;

        // assert
        assert_eq!(6, files.len());

        // cleanup
        for f in first_level_files
            .into_iter()
            .chain(second_level_files.into_iter())
        {
            drop(f);
        }

        for d in [temp_inner_dir, temp_base_dir].into_iter() {
            d.close()?;
        }

        Ok(())
    }

    #[test]
    fn valid_cpp_file_path_test() {
        let valid_path = TEST_PATH.join("file.cpp");
        assert!(ProjectScanner::is_valid_file_path(valid_path.to_str().unwrap()));
    }

    #[test]
    fn valid_header_file_path_test() {
        let valid_path = TEST_PATH.join("file.h");
        assert!(ProjectScanner::is_valid_file_path(valid_path.to_str().unwrap()));
    }

    #[test]
    fn invalid_hidden_directory_path_test() {
        let invalid_path = INVALID_TEST_PATH.join("file.h");
        assert!(!ProjectScanner::is_valid_file_path(invalid_path.to_str().unwrap()));
    }

    #[test]
    fn invalid_hidden_file_path_test() {
        let invalid_path = INVALID_TEST_PATH.join(".file.cpp");
        assert!(!ProjectScanner::is_valid_file_path(invalid_path.to_str().unwrap()));
    }

    #[test]
    fn apply_blacklist() -> Result<(), Box<dyn Error>> {
        let (temp_base_dir, temp_inner_dir) = create_dir_tree()?;

        let first_level_files = create_cpp_files_in_path(temp_base_dir.path(), vec!["file1.cpp"], vec![""])?;
        let second_level_files = create_cpp_files_in_path(temp_inner_dir.path(), vec!["file2.cpp"], vec![""])?;

        let mut project = super::ProjectScanner::make(&temp_base_dir.path())?;

        // act
        let files = project.scan_files()?;
        Ok(())
    }
}
