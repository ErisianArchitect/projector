use std::{
    path::{
        Path,
    },
    time::{
        SystemTime
    },
};
use chrono::{
    DateTime,
    Utc,
    Local,
    TimeZone,
};

pub fn modified_system_time<P: AsRef<Path>>(path: P) -> std::io::Result<SystemTime> {
    let path = path.as_ref();
    let metadata = path.metadata()?;
    let modified_time = metadata.modified()?;
    Ok(modified_time)
}

pub fn modified_time<P: AsRef<Path>, T: TimeZone>(path: P) -> std::io::Result<DateTime<T>>
where DateTime<T>: From<SystemTime> {
    Ok(modified_system_time(path)?.into())
}

#[inline]
pub fn local_modified_time<P: AsRef<Path>>(path: P) -> std::io::Result<DateTime<Local>> {
    modified_time(path.as_ref())
}

#[inline]
pub fn utc_modified_time<P: AsRef<Path>>(path: P) -> std::io::Result<DateTime<Utc>> {
    modified_time(path.as_ref())
}

/// Determines if two paths point to the same location in the file system.
/// 
/// This will canonicalize the paths then compare them.
#[inline]
pub fn is_same_path<PL: AsRef<Path>, PR: AsRef<Path>>(lhs: PL, rhs: PL) -> std::io::Result<bool> {
    let lhs = lhs.as_ref().canonicalize()?;
    let rhs = rhs.as_ref().canonicalize()?;
    Ok(lhs == rhs)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn file_time_test() -> std::io::Result<()> {
        // std::fs::write("output.txt", "The quick brown fox jumps over the lazy dog.")?;
        let mod_time = local_modified_time("output.txt")?;
        let mod_time_fmt = mod_time.format("Modified on %Y-%m-%d at %I:%M:%S%p");
        let mod_time_text = format!("{}", mod_time_fmt);
        println!("{mod_time_text}");
        Ok(())
    }
}