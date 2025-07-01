use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use tempdir::TempDir;
use walkdir::WalkDir;

mod project {
    use std::fs::read_to_string;

    use super::*;

    pub struct File {
        name: String,
        used_modules: Vec<String>,
    }

    impl File {
        pub fn make(name: &str, file_content: &str) -> Result<File, &'static str> {
            let used_modules = File::make_used_modules(file_content)?;

            Ok(File {
                name: String::from(name),
                used_modules: used_modules,
            })
        }

        pub fn get_name(&self) -> &str {
            &self.name
        }

        pub fn get_used_modules(&self) -> &[String] {
            &self.used_modules
        }

        fn make_used_modules(file_content: &str) -> Result<Vec<String>, &'static str> {
            let re = Regex::new(r#"^\s*#include\s*[<"](.*?)[>"](?:\s*//.*)?$"#)
                .map_err(|_| "Error in regex creation")?;

            let used_modules = file_content
                .lines()
                .filter(|row| !row.trim_start().starts_with("//"))
                .filter(|row| !row.trim_start().starts_with("/*"))
                .filter_map(|row| {
                    re.captures(row)
                        .and_then(|captures| captures.get(1))
                        .map(|m| m.as_str().to_string())
                })
                .collect();

            Ok(used_modules)
        }
    }

    pub struct Project<'a> {
        base_path: &'a Path,
        files: Vec<File>,
    }

    impl<'a> Project<'a> {
        pub fn make(base_path: &Path) -> Result<Project, Box<dyn Error>> {
            Ok(Project {
                base_path: base_path,
                files: Vec::new(),
            })
        }

        pub fn scan_files(&mut self) -> Result<(), Box<dyn Error>> {
            for entry in WalkDir::new(&self.base_path) {
                let entry = entry?;
                let path = entry.path();
                let file_type = entry.file_type();

                if file_type.is_file() {
                    let content = read_to_string(path)?;
                    self.files
                        .push(File::make(path.to_str().unwrap(), &content)?);
                }
            }

            Ok(())
        }

        pub fn get_num_files(&self) -> usize {
            self.files.len()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpp_file_parsing_test() -> Result<(), &'static str> {
        let file_name = "main.cpp";
        let f = project::File::make(
            file_name,
            "\
#include <iostream>
#include \"foobar.h\"
//#include \"commented_out.h\"
/*#include \"another_commented_out.h\"

int main(void) {
    printf(\"Hello world\");
    return 0;
};",
        )?;

        assert_eq!(file_name, f.get_name());
        assert_eq!(
            vec![String::from("iostream"), String::from("foobar.h")],
            f.get_used_modules()
        );

        Ok(())
    }

    fn create_cpp_files_in_path(path: &Path) -> Result<Vec<File>, Box<dyn Error>> {
        let first_path = path.join("first.cpp");
        let mut first = File::create(first_path)?;

        writeln!(
            first,
            "\
#include <iostream>
#include \"third.h\"
// #include \"some_random_header.h\"

void main() {{
    // commented out code
}}

"
        )?;

        let second_path = path.join("second.cpp");
        let mut second = File::create(second_path)?;

        writeln!(
            second,
            "\
#include \"third.h\"
#include \"very_basic_header.h\"

void foobar() {{
    // doing some internal stuff here
}}
"
        )?;

        let third_path = path.join("third.h");
        let mut third = File::create(third_path)?;

        writeln!(
            third,
            "\
#include \"some_random_header_too.h\"

class FooBar {{
    explicit FooBar() = default;

    void DoStuff() noexcept {{}};
}};

"
        )?;

        Ok(vec![first, second, third])
    }

    #[test]
    fn cpp_directory_parsing_test() -> Result<(), Box<dyn Error>> {
        // arrange
        let temp_dir = TempDir::new("scar_cpp_directory_parsing_test")
            .map_err(|_| "Temporary directory creation error")?;

        let files = create_cpp_files_in_path(temp_dir.path())?;

        let mut project = project::Project::make(&temp_dir.path())?;
        project.scan_files()?;

        // assert
        assert_eq!(3, project.get_num_files());

        // cleanup
        for f in files {
            drop(f);
        }

        temp_dir.close()?;
        Ok(())
    }
}
