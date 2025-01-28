use std::{fs, io::Read, path};

use sha2::{Digest, Sha256};

/// Hash a file as SHA256
pub fn hash_file(file: &path::Path) -> Result<String, std::io::Error> {
    let mut hasher = Sha256::new();
    let mut file = fs::File::open(file)?;
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Hash a string as SHA256
pub fn hash_string<S>(input: S) -> String
where
    S: Into<String>,
{
    let mut hasher = Sha256::new();
    hasher.update(input.into());
    let result = hasher.finalize();
    format!("{:x}", result)
}
