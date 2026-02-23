
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {

    println!("cargo:rerun-if-changed=src/models");

    // let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR not set");
    let out_dir_path =Path::new("src/models/");
    let dest_path = out_dir_path.join("mod.rs");

    let models_dir = PathBuf::from("src/models");

    let mut content = String::new();

    match fs::read_dir(&models_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() || path.extension().map_or(false, |e| e == "rs") == false {
                    continue;
                }

                let stem = match path.file_stem().and_then(|s| s.to_str()) {
                    Some(s) => s.to_string(),
                    None => continue,
                };

                if !stem.ends_with("_generated") {
                    continue;
                }

                let mod_name = stem.trim_end_matches("_generated").to_string();

                let struct_name: String = mod_name.split('_').map(|word| {
                    let mut chars = word.chars();
                    chars.next().map_or(String::new(), |c| c.to_uppercase().collect::<String>() + &chars.collect::<String>())
                }).collect();

                content.push_str(&format!("pub mod {};\n", stem));
                content.push_str(&format!("pub use {}::{};\n", stem, struct_name));
            }
        }
        Err(e) => {
            eprintln!("Warning: could not read models dir: {}", e);
        }
    }

    let mut file = File::create(&dest_path).expect("could not create all_models.rs");
    file.write_all(content.as_bytes()).expect("could not write to all_models.rs");

    println!("cargo:warning=Generated content:\n{}", content);  // برای debug
}