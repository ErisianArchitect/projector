use std::path::{
    Path, PathBuf,
};
use std::collections::HashSet;


pub struct Dirs {
    // `directories` is expected to be very small (3 items or less, rarely more).
    // That means that it won't be expensive to do move/insert/remove operations,
    // especially since those operations will be rare and non-sequential.
    directories: Vec<PathBuf>,
    set: HashSet<PathBuf>,
}

impl Dirs {
    pub fn new() -> Self {
        Self {
            directories: Vec::new(),
            set: HashSet::new(),
        }
    }

    // Operations:
    // - Move
    // - Append
    // - Remove

    pub fn move_to(&mut self, from: usize, to: usize) {
        if from == to {
            return;
        }
        if from < to {
            let to = to - 1;
            let value = self.directories.remove(from);
            self.directories.insert(to, value);
        } else {
            let value = self.directories.remove(from);
            self.directories.insert(to, value);
        }
    }

    pub fn move_up(&mut self, index: usize) {
        if index == 0 {
            return;
        }
        self.directories.swap(index, index - 1);
    }

    pub fn move_down(&mut self, index: usize) {
        if index + 1 >= self.directories.len() {
            return;
        }
        self.directories.swap(index, index + 1);
    }

    pub fn append<P: AsRef<Path> + Into<PathBuf>>(&mut self, path: P) -> bool {
        let path: PathBuf = path.into();
        if self.set.insert(path.clone()) {
            self.directories.push(path);
            true
        } else {
            false
        }
    }

    pub fn remove<P: AsRef<Path> + Into<PathBuf>>(&mut self, path: P) -> bool {
        let path: PathBuf = path.into();
        if self.set.remove(&path) {
            for i in 0..self.directories.len() {
                if self.directories[i] == path {
                    self.directories.remove(i);
                    break;
                }
            }
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn dirs_test() {
        let mut dirs = Dirs::new();
        dirs.append(r#"C:\Users\derek\Documents\code\rust\projector"#);
        dirs.append(r#"C:\Users\derek\Documents\code\rust\bourne"#);
        dirs.append(r#"C:\Users\derek\Documents\code\rust\bourne"#);
        dirs.append(r#"C:\Users\derek\Documents\code\rust\rollgrid"#);
        for (i, dir) in dirs.directories.iter().enumerate() {
            println!("{i}: {}", dir.display());
        }
        println!("****************************************************************");
        dirs.move_down(1);
        for (i, dir) in dirs.directories.iter().enumerate() {
            println!("{i}: {}", dir.display());
        }
        println!("****************************************************************");
        dirs.move_up(2);
        for (i, dir) in dirs.directories.iter().enumerate() {
            println!("{i}: {}", dir.display());
        }
        println!("****************************************************************");
        dirs.move_down(2);
        for (i, dir) in dirs.directories.iter().enumerate() {
            println!("{i}: {}", dir.display());
        }
    }
}