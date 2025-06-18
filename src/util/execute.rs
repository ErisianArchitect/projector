use std::process::*;

#[derive(Debug, thiserror::Error)]
pub enum ExecError {
    #[error("Io Error: {0}")]
    IoError(#[from] std::io::Error),
}

pub fn execute<S: AsRef<str>>(script: S) -> Result<ExitStatus, std::io::Error> {
    #[inline]
    fn inner(script: &str) -> Result<ExitStatus, std::io::Error> {
        #[cfg(target_os = "windows")]
        {
            use std::{io::Write, os::windows::process::CommandExt};
    
            let mut temp = tempfile::NamedTempFile::with_suffix(".bat")?;
            temp.write_all(script.as_bytes())?;
            temp.flush()?;
            let script_path = temp.path();
            let path_str = script_path.display().to_string();
            let status = Command::new("cmd")
                .args(["/C", &path_str])
                .creation_flags(0x08000000) // prevent creation of window.
                .status()?;
            // Prevents premature drop.
            drop(temp);
            Ok(status)
        }
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            use std::io::Write;
    
            let mut temp = tempfile::NamedTempFile::with_suffix(".sh")?;
            temp.write_all(script.as_bytes())?;
            temp.flush()?;
            let script_path = temp.path();
            let path_str = script_path.display().to_string();
            let status = Command::new("sh")
                .arg(&path_str)
                .status()?;
            // Prevents premature drop.
            drop(temp);
            Ok(status)
        }
    }
    inner(script.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn execute_test() {
        execute("code run.bat").expect("Failed to execute.");
    }
}