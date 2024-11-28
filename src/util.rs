use std::env;
use std::error::Error;
use std::path::PathBuf;

pub fn get_path(path: &str) -> Result<PathBuf, Box<dyn Error>> {
    let file_path = env::current_exe()?
        .parent()
        .ok_or("Failed to get parent directory")?
        .to_path_buf()
        .join(path);
    Ok(file_path)
}
