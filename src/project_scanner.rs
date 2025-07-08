use crate::file::File;
use lazy_static;
use std::collections::HashMap;
use std::error::Error;
use std::fs::read_to_string;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

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
                String::from("TestAutomationCore"),
                String::from("Binaries"),
                String::from("TestData"),
                String::from("generated.h"),
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
                        files.push(File::make(entry.file_name().to_str().unwrap(), &content)?);

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
        let is_path_valid = Self::is_valid_file_path(entry.path().to_str().unwrap());
        is_path_valid
            && (entry.file_type().is_dir()
                || entry
                    .file_name()
                    .to_str()
                    .map(|s| Self::is_valid_file_name(s))
                    .unwrap_or(false))
    }

    fn is_valid_file_name(path: &str) -> bool {
        !path.starts_with(".") && (path.ends_with(".cpp") || path.ends_with(".h"))
    }

    fn is_valid_file_path(path: &str) -> bool {
        !Self::is_blacklisted(path)
    }

    fn on_processed_file(&mut self) {
        self.processed_files += 1;
        if self.processed_files > 0 && self.processed_files % 1000 == 0 {
            println!("Processed num. files: {}", self.processed_files);
        }
    }

    fn is_blacklisted(entry: &str) -> bool {
        let is_blacklisted = CONFIG.get("black_list").map_or(false, |black_list| {
            black_list.iter().any(|bl| entry.contains(bl))
        });

        is_blacklisted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use std::path::PathBuf;
    use tempdir::TempDir;

    static FIRST_TEST_CONTENT: &str = "#include \"third.h\"
        #include \"very_basic_header.h\"
        
        void foobar() {{
            // doing some internal stuff here
            }}";

    static SECOND_TEST_CONTENT: &str = "#include <iostream>
            #include \"third.h\"
            // #include \"some_random_header.h\"

            void main() {{
                // commented out code
            }}";

    static THIRD_TEST_CONTENT: &str = "
    #include \"some_random_header_too.h\"
    
    class FooBar {{
        explicit FooBar() = default;
        
        void DoStuff() noexcept {{}};
        }};";

    lazy_static! {
        static ref TEST_PATH: PathBuf = PathBuf::from("/media/workspace");
        static ref INVALID_TEST_PATH: PathBuf = PathBuf::from(".media/workspace/");

        static ref TEST_PATH_TO_BE_FILTERED: Vec<PathBuf> = vec![
            PathBuf::from("/media/workspace/Source/Intermediate/Plugins/Binaries/test.cpp"),
            PathBuf::from("/media/workspace/Source/Intermediate/Plugins/Binaries/SomePlugin/test.h"),
            PathBuf::from("/media/workspace/repos/BarFoo/FooBar/Intermediate/Build/Linux/UnrealEditor/Inc/KitchenEntities/UHT/KEKitchenMaterialDataC.generated.h"),
            PathBuf::from("/media/workspace/repos/BarFoo/FooBar/Plugins/SERE/Source/SimpleElementsRenderingExtension/Shaders"),
            PathBuf::from("/home/user/repos/BarFoo/FooBar/Plugins/USQLite/Source/Runtime/Public/USQLReflector.h"),
            PathBuf::from("/home/user/repos/BarFoo/FooBar/Plugins/USQLite/Source/Runtime/Public/USQLReflector.generated.h"),
        ];

        static ref TEST_PATH_NOT_TO_BE_FILTERED: Vec<PathBuf> = vec![
            PathBuf::from("/media/workspace/Source/test.cpp"),
            PathBuf::from("/media/workspace/Source/test.h"),
        ];
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

    fn create_cpp_files_in_path(
        path: &Path,
        files: Vec<&str>,
        contents: Vec<&str>,
    ) -> Result<Vec<File>, Box<dyn Error>> {
        assert!(
            files.len() == contents.len(),
            "Files and contents must have the same length"
        );
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

        let first_level_files = create_cpp_files_in_path(
            temp_base_dir.path(),
            vec!["first.cpp", "second.cpp", "third.h"],
            vec![FIRST_TEST_CONTENT, SECOND_TEST_CONTENT, THIRD_TEST_CONTENT],
        )?;
        let second_level_files = create_cpp_files_in_path(
            temp_inner_dir.path(),
            vec!["first.cpp", "second.cpp", "third.h"],
            vec![FIRST_TEST_CONTENT, SECOND_TEST_CONTENT, THIRD_TEST_CONTENT],
        )?;

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
        assert!(ProjectScanner::is_valid_file_name(
            valid_path.to_str().unwrap()
        ));
    }

    #[test]
    fn valid_header_file_path_test() {
        let valid_path = TEST_PATH.join("file.h");
        assert!(ProjectScanner::is_valid_file_name(
            valid_path.to_str().unwrap()
        ));
    }

    #[test]
    fn invalid_hidden_directory_path_test() {
        let invalid_path = INVALID_TEST_PATH.join("file.h");
        assert!(!ProjectScanner::is_valid_file_name(
            invalid_path.to_str().unwrap()
        ));
    }

    #[test]
    fn invalid_hidden_file_path_test() {
        let invalid_path = INVALID_TEST_PATH.join(".file.cpp");
        assert!(!ProjectScanner::is_valid_file_name(
            invalid_path.to_str().unwrap()
        ));
    }

    #[test]
    fn blacklisted_directory_path_test() {
        for path in TEST_PATH_TO_BE_FILTERED.iter() {
            assert!(ProjectScanner::is_blacklisted(path.to_str().unwrap()));
        }

        for path in TEST_PATH_NOT_TO_BE_FILTERED.iter() {
            assert!(!ProjectScanner::is_blacklisted(path.to_str().unwrap()));
        }
    }
}
