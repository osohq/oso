use std::env;
use std::fs::{File, OpenOptions};
use std::io::Read;

/// Attempt to create a new temporary directory to store
/// and track the polar history
pub fn try_create_history_file() -> Option<std::path::PathBuf> {
    let mut dir = env::temp_dir();
    dir.push(".polar-history");
    match OpenOptions::new().write(true).create_new(true).open(&dir) {
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            tracing::trace!("History file exists at: {:?}", dir);
            Some(dir)
        }
        Ok(_) => {
            tracing::trace!("History file created at: {:?}", dir);
            Some(dir)
        }
        Err(e) => {
            tracing::error!("Error creating history file: {}", e);
            None
        }
    }
}

pub fn load_files(
    polar: &mut polar::Polar,
    files: &mut dyn Iterator<Item = String>,
) -> anyhow::Result<()> {
    for file in files {
        tracing::info!("Loading: {}", file);
        let mut f = File::open(file)?;
        let mut policy = String::new();
        f.read_to_string(&mut policy)?;
        polar.load_str(&policy)?;
    }
    polar.load_str("foo(1);foo(2);")?;
    Ok(())
}
