use std::{borrow::Borrow, collections::HashSet, hash::Hash, path::{Path, PathBuf}};

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, bincode::Decode, bincode::Encode)]
pub enum ProjectPath {
    Rust(PathBuf),
    Python(PathBuf),
    Web(PathBuf),
    Other(PathBuf),
}

#[derive(Debug, Clone, bincode::Decode, bincode::Encode)]
pub struct ProjectDirs {
    directories: Vec<PathBuf>,
    set: HashSet<PathBuf>,
}

impl ProjectDirs {
    #[inline]
    pub fn new() -> Self {
        Self {
            directories: Vec::new(),
            set: HashSet::new(),
        }
    }

    /// Inserts a directory into the collection and returns true if it was newly inserted. Returns false if it was already in the collection.
    pub fn insert<P: Into<PathBuf>>(&mut self, directory: P) -> bool {
        #[inline]
        fn inner(dirs: &mut ProjectDirs, directory: PathBuf) -> bool {
            if dirs.set.insert(directory.clone()) {
                dirs.directories.push(directory);
                true
            } else {
                false
            }
        }
        inner(self, directory.into())
    }

    pub fn remove<P>(&mut self, path: P) -> bool
    where
        P: AsRef<Path> + Into<PathBuf>,
    {
        if self.set.remove(path.as_ref()) {
            let mut rem_index = None;
            for i in 0..self.directories.len() {
                if self.directories[i] == path.as_ref() {
                    rem_index = Some(i);
                    break;
                }
            }
            if let Some(index) = rem_index {
                self.directories.remove(index);
            }
            true
        } else {
            false
        }
    }

    pub fn contains<P: AsRef<Path>>(&self, path: P) -> bool {
        self.set.contains(path.as_ref())
    }

    #[inline]
    pub fn directories(&self) -> &[PathBuf] {
        &self.directories
    }
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

    #[test]
    fn project_dirs_test() {
        let mut proj_dirs = ProjectDirs::new();
        if proj_dirs.insert(r#"C:\Users\derek\Documents\code\rust\projector"#) {
            println!("Inserted projector.");
        }

        if proj_dirs.insert(r#"C:\Users\derek\Documents\code\rust\projector"#) {
            println!("Inserted projector.");
        } else {
            println!("Did not insert projector again.");
        }

        if proj_dirs.contains(r#"C:\Users\derek\Documents\code\rust\projector"#) {
            println!("Contains projector");
        } else {
            println!("Does not contain projector");
        }

        if proj_dirs.remove(r#"C:\Users\derek\Documents\code\rust\projector"#) {
            println!("Projector removed");
        } else {
            println!("Projector was not removed");
        }

        if proj_dirs.contains(r#"C:\Users\derek\Documents\code\rust\projector"#) {
            println!("Contains projector");
        } else {
            println!("Does not contain projector");
        }

        if proj_dirs.remove(r#"C:\Users\derek\Documents\code\rust\projector"#) {
            println!("Projector removed");
        } else {
            println!("Projector was not removed");
        }
    }
}