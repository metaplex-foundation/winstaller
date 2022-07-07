use anyhow::Result;
use mktemp::Temp;
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

const DOWNLOAD_PATH: &str =
    "https://github.com/metaplex-foundation/sugar/releases/latest/download/sugar-macos-m1-latest";

#[tokio::main]
async fn main() -> Result<()> {
    let home = if cfg!(unix) {
        println!("For Unix systems use the install script in the Sugar README.");
        std::process::exit(1);
    } else if cfg!(windows) {
        let drive = env::var_os("HOMEDRIVE").expect("Coulnd't find Windows home drive key.");
        let path = env::var_os("HOMEPATH").expect("Coulnd't find Windows home path key.");
        Path::new(&drive).join(&path).as_os_str().to_owned()
    } else if cfg!(target_os = "macos") {
        println!("For MacOS systems use the install script in the Sugar README.");
        std::process::exit(1);
    } else {
        panic!("Unsupported OS!");
    };

    let cargo_bin_path = Path::new(&home).join(".cargo").join("bin");
    let program_files_path = Path::new(&home).join("Program Files");

    // Create temporary directory for downloaded binary. `mktemp` drops the file when the variable goes out of scope.
    let temp = Temp::new_file()?;
    let temp_file_path = temp.to_path_buf();
    println!("Temp file: {:?}", temp_file_path);

    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&temp_file_path)?;

    println!("Getting binary....");
    let contents = reqwest::get(DOWNLOAD_PATH).await?.bytes().await?;
    println!("Writing binary....");
    f.write_all(&contents)?;

    // Prefer to install to .cargo/bin if it exists, otherwise use Program Files.
    if cargo_bin_path.exists() {
        println!("Installing to .cargo/bin...");
        fs::copy(&temp_file_path, cargo_bin_path.join("sugar.exe"))?;
    } else {
        println!("Installing to Program Files...");
        fs::copy(&temp_file_path, program_files_path.join("sugar.exe"))?;
    }

    Ok(())
}
