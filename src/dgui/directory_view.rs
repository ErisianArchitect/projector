use std::path::{Path, PathBuf};



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathType {
    Files,
    Directories,
    FilesAndDirectories,
}

impl PathType {
    pub fn path_is_type<P: AsRef<Path>>(self, path: P) -> bool {
        fn inner(ty: PathType, path: &Path) -> bool {
            match ty {
                PathType::Files => path.is_file(),
                PathType::Directories => path.is_dir(),
                PathType::FilesAndDirectories => path.is_file() || path.is_dir(),
            }
        }
        inner(self, path.as_ref())
    }
}

#[derive(Debug)]
pub struct PopulateError {
    pub view: DirectoryView,
    pub err: std::io::Error,
}

#[derive(Debug)]
pub struct DirectoryView {
    root: PathBuf,
    path_types: PathType,
    entries: Vec<PathBuf>,
}

impl DirectoryView {
    pub fn new<P: Into<PathBuf>>(root: P, path_types: PathType) -> Self {
        Self {
            root: root.into(),
            path_types,
            entries: Vec::new(),
        }
    }

    pub fn and_populate(mut self) -> Result<Self, PopulateError> {
        match self.refresh() {
            Ok(()) => Ok(self),
            Err(err) => Err(PopulateError {
                view: self,
                err,
            }),
        }
    }

    pub fn refresh(&mut self) -> std::io::Result<()> {
        self.entries.clear();
        for entry in self.root.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if self.path_types.path_is_type(&path) {
                self.entries.push(path);
            }
        }
        Ok(())
    }
}