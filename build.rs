use std::fs::File;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

const NNUE_FILE_NAME: &str = "nn-62ef826d1a6d.nnue";
// const NNUE_FILE_NAME: &str = "nn-c3ca321c51c9.nnue";

struct NNUEGenerationError(String);

impl<T: ToString> From<T> for NNUEGenerationError {
    fn from(msg: T) -> Self {
        NNUEGenerationError(msg.to_string())
    }
}

impl std::fmt::Debug for NNUEGenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn remove_nnue_file(nnue_path: &Path) -> Result<(), NNUEGenerationError> {
    if nnue_path.is_file() {
        let err_msg = format!(
            "Could not delete file {}!",
            nnue_path.to_str().ok_or("NNUE Path not found")?
        );
        std::fs::remove_file(nnue_path).map_err(|_| err_msg)?;
    }
    Ok(())
}

fn nnue_downloaded_correctly(nnue_path: &Path) -> bool {
    if !nnue_path.is_file() {
        return false;
    }
    let expected_hash_start = NNUE_FILE_NAME
        .strip_prefix("nn-")
        .unwrap()
        .strip_suffix(".nnue")
        .unwrap();
    let nnue_data = std::fs::read(nnue_path).unwrap();
    let hash = sha256::digest(nnue_data.as_slice());
    hash.starts_with(expected_hash_start)
}

fn generate_nnue_file(nnue_file: &mut File) -> Result<(), NNUEGenerationError> {
    let nnue_file_link = format!("https://tests.stockfishchess.org/api/nn/{}", NNUE_FILE_NAME);
    reqwest::blocking::get(nnue_file_link)
        .map_err(|_| "Could not download NNUE file! Check your internet connection!")?
        .copy_to(nnue_file)
        .map_err(|_| "Could not copy NNUE file data to the nnue file!")?;
    Ok(())
}

fn main() {
    let output_dir = std::env::var("OUT_DIR").unwrap();
    let nnue_dir = Path::new(&output_dir).join("nnue_dir");
    if !nnue_dir.is_dir() {
        std::fs::create_dir(nnue_dir.clone()).unwrap();
    }
    let nnue_path = nnue_dir.join("nn.nnue");
    if !nnue_downloaded_correctly(&nnue_path) {
        remove_nnue_file(&nnue_path).unwrap();
        let mut nnue_file = File::create(nnue_path.clone()).expect("failed to create file");
        match generate_nnue_file(&mut nnue_file) {
            Ok(_) => {
                println!("cargo:rerun-if-changed={}", nnue_path.to_str().unwrap());
            }
            Err(err) => {
                remove_nnue_file(&nnue_path).unwrap();
                panic!("{err:?}");
            }
        }
        if nnue_file.metadata().unwrap().size() < 2_u64.pow(10) {
            panic!("File not downloaded correctly!");
        }
    }
}
