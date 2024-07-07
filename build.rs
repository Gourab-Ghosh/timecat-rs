pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[cfg(feature = "inbuilt_nnue")]
mod nnue_features {
    use super::*;
    pub use std::fs::File;
    pub use std::path::{Path, PathBuf};

    pub const NNUE_FILE_NAME: &str = "nn-62ef826d1a6d.nnue";
    // pub const NNUE_FILE_NAME: &str = "nn-f7d87b7a1789.nnue";
    // const NNUE_FILE_NAME: &str = "nn-c3ca321c51c9.nnue";

    pub fn remove_nnue_file(nnue_path: &Path) -> Result<()> {
        if nnue_path.is_file() {
            let err_msg = format!(
                "Could not delete file {}!",
                nnue_path.to_str().ok_or("NNUE Path not found")?
            );
            std::fs::remove_file(nnue_path).map_err(|_| err_msg)?;
        }
        Ok(())
    }

    pub fn nnue_downloaded_correctly(nnue_path: &Path) -> Result<bool> {
        if !nnue_path.is_file() {
            return Ok(false);
        }
        let expected_hash_start = NNUE_FILE_NAME
            .strip_prefix("nn-")
            .unwrap()
            .strip_suffix(".nnue")
            .unwrap();
        let nnue_data = std::fs::read(nnue_path)?;
        let hash = sha256::digest(nnue_data.as_slice());
        Ok(hash.starts_with(expected_hash_start))
    }

    pub fn generate_nnue_file(nnue_file: &mut File) -> Result<()> {
        let nnue_file_link = format!("https://tests.stockfishchess.org/api/nn/{}", NNUE_FILE_NAME);
        reqwest::blocking::get(nnue_file_link)
            .map_err(|_| "Could not download NNUE file! Check your internet connection!")?
            .copy_to(nnue_file)
            .map_err(|_| "Could not copy NNUE file data to the nnue file!")?;
        Ok(())
    }

    pub fn check_and_download_nnue(nnue_dir: &PathBuf) -> Result<()> {
        if !nnue_dir.is_dir() {
            std::fs::create_dir_all(nnue_dir.clone())?;
        }
        let nnue_path = nnue_dir.join("nn.nnue");
        if !nnue_downloaded_correctly(&nnue_path)? {
            remove_nnue_file(&nnue_path)?;
            let mut nnue_file = File::create(nnue_path.clone())
                .map_err(|_| format!("Failed to create file at {:?}", nnue_dir))?;
            println!("cargo:rerun-if-env-changed=DOCS_RS");
            println!("cargo:rerun-if-env-changed=NNUE_DOWNLOAD");
            if std::env::var("DOCS_RS").is_ok()
                || std::env::var("NNUE_DOWNLOAD") == Ok("PAUSE".to_string())
            {
                return Ok(());
            }
            match generate_nnue_file(&mut nnue_file) {
                Ok(_) => {
                    println!("cargo:rerun-if-changed={:?}", nnue_path);
                }
                Err(err) => {
                    remove_nnue_file(&nnue_path)?;
                    return Err(err);
                }
            }
            if !nnue_downloaded_correctly(&nnue_path)? {
                return Err("File not downloaded correctly!".into());
            }
        }
        Ok(())
    }
}

#[cfg(feature = "inbuilt_nnue")]
fn main() -> Result<()> {
    use nnue_features::*;

    let output_dir = std::env::var("OUT_DIR")?;
    let output_nnue_dir = Path::new(&output_dir).join("nnue_dir");
    // Backing up nnue file in local cache directory to prevent downloading it multiple times
    let nnue_dir = dirs::cache_dir()
        .map(|path| path.join("timecat").join("nnue_dir"))
        .unwrap_or(output_nnue_dir.clone());
    match check_and_download_nnue(&nnue_dir) {
        Ok(_) => {
            if nnue_dir != output_nnue_dir {
                std::fs::create_dir_all(output_nnue_dir.clone())?;
                std::fs::copy(nnue_dir.join("nn.nnue"), output_nnue_dir.join("nn.nnue"))?;
            }
        }
        Err(err) => {
            if nnue_dir == output_nnue_dir {
                return Err(err.into());
            } else {
                check_and_download_nnue(&nnue_dir)?;
            }
        }
    }
    Ok(())
}

#[cfg(not(feature = "inbuilt_nnue"))]
fn main() -> Result<()> {
    Ok(())
}
