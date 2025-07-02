use regex::Regex;
use std::error::Error;
use std::fs::read_to_string;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

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
        for entry in walker.filter_entry(|e| Self::is_dir_or_cpp_file(e)) {
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

    fn is_dir_or_cpp_file(entry: &DirEntry) -> bool {
        entry.file_type().is_dir()
            || entry
                .file_name()
                .to_str()
                .map(|s| s.ends_with(".cpp") || s.ends_with(".h"))
                .unwrap_or(false)
    }

    fn on_processed_file(&mut self) {
        self.processed_files += 1;
        if self.processed_files > 0 && self.processed_files % 1000 == 0 {
            println!("Processed num. files: {}", self.processed_files);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempdir::TempDir;

    #[test]
    fn cpp_file_parsing_test() -> Result<(), &'static str> {
        let file_name = "main.cpp";
        let f = super::File::make(
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

    fn create_cpp_files_in_path(path: &Path) -> Result<Vec<File>, Box<dyn Error>> {
        let first_content = "\
#include <iostream>
#include \"third.h\"
// #include \"some_random_header.h\"

void main() {{
    // commented out code
}}

";

        let second_content = "\
#include \"third.h\"
#include \"very_basic_header.h\"

void foobar() {{
    // doing some internal stuff here
}}
";

        let third_content = "\
#include \"some_random_header_too.h\"

class FooBar {{
    explicit FooBar() = default;

    void DoStuff() noexcept {{}};
}};

";

        let first = create_file(path, "first.cpp", first_content)?;
        let second = create_file(path, "second.cpp", second_content)?;
        let third = create_file(path, "third.h", third_content)?;

        Ok(vec![first, second, third])
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

        let first_level_files = create_cpp_files_in_path(temp_base_dir.path())?;
        let second_level_files = create_cpp_files_in_path(temp_inner_dir.path())?;

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
}
