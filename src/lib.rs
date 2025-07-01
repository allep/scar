mod project {
    pub struct File {
        name: String,
        used_symbols: Vec<String>,
        defined_symbols: Vec<String>,
    }

    pub struct Directory {
        name: String,
        files: Vec<File>,
        directories: Vec<Box<Directory>>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_moves_creation_test() {
        assert_eq!(14, 15);
    }
}
