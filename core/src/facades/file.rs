use std::fs;
use std::path::Path;
#[derive(Debug)]
pub struct File ;

impl File {
    /// Create a symbolic link between source and destination paths
    ///
    /// # Arguments
    ///
    /// * `source` - The source path to link from
    /// * `destination` - The destination path to link to
    ///
    /// # Examples
    ///
    /// ```rust
    /// fn main() -> std::io::Result<()> {
    ///     File::create_link("storage/app/public", "public/storage")?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an `io::Error` if the link creation fails
    pub fn create_link(source: &str, destination: &str) -> std::io::Result<()> {
        let source_path = Path::new(source);
        let destination_path = Path::new(destination);

        // check if file exists before
        if destination_path.exists() {
            fs::remove_file(destination_path)?;
        }

        // create symbolic link
        #[cfg(unix)]
        std::os::unix::fs::symlink(source_path, destination_path)?;

        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(source_path, destination_path)?;

        Ok(())
    }
}