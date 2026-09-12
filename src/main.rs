use std::env;
use std::path::PathBuf;

fn find_modloader_ini() -> Option<PathBuf> {
    let exe_path = env::current_exe().ok()?;
    let mut current_dir = exe_path.parent()?.to_path_buf();

    loop {
        let modloader_ini = current_dir.join("modloader").join("modloader.ini");

        if modloader_ini.is_file() {
            return Some(modloader_ini);
        }

        if !current_dir.pop() {
            return None;
        }
    }
}

fn main() {
    match find_modloader_ini() {
        Some(path) => {
            println!("Found modloader.ini:");
            println!("{}", path.display());
        }
        None => {
            println!("Could not find modloader\\modloader.ini");
        }
    }
}