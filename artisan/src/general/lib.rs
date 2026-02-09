use rand::RngCore;
use data_encoding::BASE64;
use std::fs;
use rand::rngs::OsRng;
use std::path::PathBuf;

pub fn generate_laravel_app_key() -> String {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);

    format!("base64:{}", BASE64.encode(&key))
}

pub fn set_env_value(key: &str, value: &str) -> std::io::Result<()> {
    let env_path: PathBuf = std::env::current_dir()?.join(".env");

    let mut content = if env_path.exists() {
        fs::read_to_string(&env_path)?
    } else {
        String::new()
    };

    let mut found = false;

    content = content
        .lines()
        .map(|line| {
            if line.starts_with(&format!("{}=", key)) {
                found = true;
                format!("{}={}", key, value)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    if !found {
        if !content.is_empty() {
            content.push('\n');
        }
        content.push_str(&format!("{}={}", key, value));
    }

    fs::write(env_path, content)
}
