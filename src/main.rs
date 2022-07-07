use anyhow::Result;
use mktemp::Temp;
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

const DOWNLOAD_PATH: &str =
    "https://github.com/metaplex-foundation/sugar/releases/latest/download/sugar-windows-latest.exe";

fn main() -> Result<()> {
    if !(cfg!(windows)) {
        println!("For Linux and MacOS systems use the install script in the Sugar README.");
        std::process::exit(1);
    }

    let drive = env::var_os("HOMEDRIVE").expect("Coulnd't find Windows home drive key.");
    let path = env::var_os("HOMEPATH").expect("Coulnd't find Windows home path key.");
    let home = Path::new(&drive).join(&path).as_os_str().to_owned();

    let cargo_bin_path = Path::new(&home).join(".cargo").join("bin");
    let program_files_path = Path::new(&drive).join(r"\Program Files");

    // Create temporary directory for downloaded binary. `mktemp` drops the file when the variable goes out of scope.
    let temp = Temp::new_file()?;
    let temp_file_path = temp.to_path_buf();

    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&temp_file_path)?;

    println!("Getting binary....");
    let contents = reqwest::blocking::get(DOWNLOAD_PATH)?.bytes()?;
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
