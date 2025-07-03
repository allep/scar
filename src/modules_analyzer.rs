use crate::file::File;
use std::collections::HashMap;
use std::error::Error;

pub struct ModulesAnalyzer<'a> {
    modules_inclusion: HashMap<&'a str, Vec<&'a str>>,
}

impl<'a> ModulesAnalyzer<'a> {
    pub fn make(files: &'a [File]) -> Result<ModulesAnalyzer, Box<dyn Error>> {
        todo!()
    }

    pub fn extract_filename_from_path(path: &str) -> &str {
        match path.split("/").last() {
            Some(last_token) => last_token,
            None => path,
        }
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

        Ok(vec![first, second, third])
    }

    #[ignore]
    #[test]
    fn simple_parsing_test() -> Result<(), Box<dyn Error>> {
        let files = create_sample_files()?;

        let analyzer = ModulesAnalyzer::make(&files)?;

        Ok(())
    }

    #[test]
    fn path_parsing_test_simple() {
        let simple_path = "include/foobar.h";
        assert_eq!(
            "foobar.h",
            ModulesAnalyzer::extract_filename_from_path(simple_path)
        );
    }

    #[test]
    fn path_parsing_test_single_token() {
        let simple_path = "foobar.h";
        assert_eq!(
            "foobar.h",
            ModulesAnalyzer::extract_filename_from_path(simple_path)
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
            ModulesAnalyzer::extract_filename_from_path(simple_path)
        );
    }
}
