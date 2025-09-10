use std::fs;
use std::path::{Path, PathBuf};

fn copy_dir_recursive(source_path: &Path, dest_path: &Path) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(source_path)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dest_path.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }
    Ok(())
}

pub fn backup_files(source_path: &PathBuf, dest_path: &PathBuf) -> Result<(), std::io::Error> {
    println!("Backing up files...");
    copy_dir_recursive(source_path, dest_path)?;
    println!("Backup completed successfully.");
    Ok(())
}
