use std::env;
use std::fs::{File, OpenOptions};
use std::io::Read;

pub mod repl;

pub fn load_files(
    polar: &mut crate::Polar,
    files: &mut dyn Iterator<Item = String>,
) -> anyhow::Result<()> {
    for file in files {
        let mut f = File::open(file)?;
        let mut policy = String::new();
        f.read_to_string(&mut policy)?;
        polar.load(&policy)?;
    }
    Ok(())
}

/// Attempt to create a new temporary directory to store
/// and track the polar history
pub fn try_create_history_file() -> Option<std::path::PathBuf> {
    let mut dir = env::temp_dir();
    dir.push(".polar-history");
    match OpenOptions::new().write(true).create_new(true).open(&dir) {
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Some(dir),
        Ok(_) => Some(dir),
        Err(e) => {
            eprintln!("Error creating history file: {}", e);
            None
        }
    }
}
