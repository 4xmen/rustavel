// use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// A struct representing file operations.
pub struct FileContent;

impl FileContent {
    /// Asynchronously writes content to a file.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice that holds the path of the file.
    /// * `content` - A string slice that holds the content to be written to the file.
    ///
    /// # Returns
    ///
    /// Returns a `Result<bool, io::Error>`. `Ok(true)` indicates the content was successfully written to the file,
    /// while an error will return the error value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustavel_core::facades::file_content::FileContent;
    /// async fn run()  {
    ///     FileContent::put("example.txt","hello, world!").await;
    /// }
    /// ```
    pub async fn put(path: &str, content: &str) -> Result<bool, io::Error> {
        let path = Path::new(path);

        // Open or create the file for writing.
        let mut file = match File::create(path).await {
            Ok(file) => file,
            Err(e) => return Err(e),
        };

        // Write the content to the file.
        match file.write_all(content.as_bytes()).await {
            Ok(_) => Ok(true),
            Err(e) => Err(e),
        }
    }

    /// Asynchronously reads the content of a file.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice that holds the path of the file.
    ///
    /// # Returns
    ///
    /// Returns a `Result<bool, io::Error>`. `Ok(true)` indicates the content was successfully read from the file,
    /// while an error will return the error value.
    ///
    /// The file's content is printed to the console.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rustavel_core::facades::file_content::FileContent;
    /// async fn run()  {
    ///     let txt = FileContent::get("example.txt").await;
    /// }
    /// ```
    pub async fn get(path: &str) -> Result<bool, io::Error> {
        let path = Path::new(path);

        // Open the file for reading.
        let mut file = match File::open(path).await {
            Ok(file) => file,
            Err(e) => return Err(e),
        };

        // Read the content of the file into a string.
        let mut contents = String::new();
        match file.read_to_string(&mut contents).await {
            Ok(_) => {
                println!("File contents: {}", contents); // Print the file content to the console
                Ok(true)
            }
            Err(e) => Err(e),
        }
    }
}