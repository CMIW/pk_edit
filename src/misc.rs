use std::fs::File;
use std::io::Write;

const DB: &[u8] = include_bytes!("../pk_edit.db");

/// Extracts the bundled SQLite database to `./pk_edit.db` in the current working directory.
///
/// Uses [`File::create_new`] so it is a no-op if the file already exists (returns an error that
/// the caller should ignore). Must be called before any database query function.
///
/// # Errors
/// Returns an error if the file cannot be created or written to.
pub fn extract_db() -> std::io::Result<()> {
    let mut f = File::create_new("./pk_edit.db")?;
    f.write_all(DB)?;
    Ok(())
}
