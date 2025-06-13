use std::path::{Path, PathBuf};

pub fn populate_entries_into<P: AsRef<Path>>(directory: P, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    fn inner(directory: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
        let read_dir = std::fs::read_dir(directory)?;
        for entry in read_dir {
            let Ok(entry) = entry else {
                continue;
            };
            out.push(entry.path().to_owned());
        }
        Ok(())
    }
    inner(directory.as_ref(), out)
}

pub fn populate_entries<P: AsRef<Path>>(directory: P) -> std::io::Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    populate_entries_into(directory.as_ref(), &mut entries)?;
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn populate_directory() {
        let directory = rfd::FileDialog::new().pick_folder().expect("You didn't pick a folder, dumbass.");
        let entries = populate_entries(directory).expect("Failed to populate entries");
        for entry in entries {
            println!("{}", entry.file_name().unwrap().to_string_lossy());
        }
    }
}