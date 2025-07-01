use regex::Regex;

mod project {

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
            let re = super::Regex::new(r#"^\s*#include\s*[<"](.*?)[>"](?:\s*//.*)?$"#)
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
}
