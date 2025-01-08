use std::{fs::File, io::Read, path::Path};

/// Load bytecode file from gs2bc directory and return the reader.
///
/// # Arguments
/// - `name`: The name of the file to load.
///
/// # Returns
/// - A `Result` containing the reader if the file was found, or an error if it was not.
pub fn load_bytecode(name: &str) -> Result<impl Read, std::io::Error> {
    let path = Path::new("tests").join("gs2bc").join(name);
    let file = File::open(path)?;
    Ok(file)
}

/// Load expected output file from gs2bc directory and return the reader.
///
/// # Arguments
/// - `name`: The name of the file to load.
///
/// # Returns
/// - A `Result` containing a string with the contents of the file if it was found, or an error if it was not.
pub fn load_expected_output(name: &str) -> Result<String, std::io::Error> {
    let path = Path::new("tests").join("expected_output").join(name);
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Gets a list of all the files in the gs2bc directory.
///
/// # Returns
/// - A `Result` containing a vector of strings with the names of the files in the directory, or an error if the directory could not be read.
pub fn get_all_bytecode_files() -> Result<Vec<String>, std::io::Error> {
    let path = Path::new("tests").join("gs2bc");
    let mut files = vec![];
    for entry in path.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name() {
                if let Some(name) = name.to_str() {
                    files.push(name.to_string());
                }
            }
        }
    }
    Ok(files)
}
