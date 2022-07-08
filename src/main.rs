use anyhow::Result;
use mktemp::Temp;
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    ptr,
};
use winapi::{
    shared::minwindef::*,
    um::winuser::{SendMessageTimeoutA, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE},
};
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

const DOWNLOAD_PATH: &str =
    "https://github.com/metaplex-foundation/sugar/releases/latest/download/sugar-windows-latest.exe";

fn main() -> Result<()> {
    if !cfg!(windows) {
        println!("For Linux and MacOS systems use the install script in the Sugar README.");
        std::process::exit(1);
    }

    let drive = env::var_os("HOMEDRIVE").expect("Coulnd't find Windows home drive key.");
    let path = env::var_os("HOMEPATH").expect("Coulnd't find Windows home path key.");
    let local_app_data = env::var_os("LOCALAPPDATA").expect("Coudln't find LOCALAPPDATA path key.");
    let home = Path::new(&drive).join(&path).as_os_str().to_owned();

    let cargo_bin_path = Path::new(&home).join(".cargo").join("bin");
    let local_programs_path = Path::new(&local_app_data).join("SugarCLI");

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

    // Make SugarCLI folder in LOCALAPPDATA if it doesn't already exist.
    if !local_programs_path.exists() {
        fs::create_dir(&local_programs_path)?;
    }

    // Add to PATH.
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (env, _) = hkcu.create_subkey("Environment")?;
    let mut path: String = env.get_value("path")?;
    path.push(';');
    path.push_str(local_programs_path.to_str().unwrap());
    env.set_value("path", &path)?;

    // Signal other processes to update their environments so the new path is registered.
    unsafe {
        SendMessageTimeoutA(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0 as WPARAM,
            "Environment\0".as_ptr() as LPARAM,
            SMTO_ABORTIFHUNG,
            5000,
            ptr::null_mut(),
        );
    }

    // Prefer to install to .cargo/bin if it exists, otherwise use LOCALAPPDATA.
    if cargo_bin_path.exists() {
        println!("Installing to .cargo/bin...");
        fs::copy(&temp_file_path, cargo_bin_path.join("sugar.exe"))?;
    } else {
        println!("Installing to Program Files...");
        fs::copy(&temp_file_path, local_programs_path.join("sugar.exe"))?;
    }

    Ok(())
}
